mod cache;
mod config;
mod github;
mod metrics;

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use cache::MetricsCache;
use config::AppConfig;
use futures::stream::{self, StreamExt};
use github::GitHubClient;
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use github::RepoId;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    version: &'static str,
}

/// Shared application state accessible to all request handlers.
struct AppState {
    /// Thread-safe client for interacting with the GitHub API.
    github_client: GitHubClient,
    /// In-memory cache for repository metrics to avoid redundant API calls and processing.
    metrics_cache: MetricsCache,
    /// Application configuration loaded from environment variables.
    config: AppConfig,
}

#[tokio::main]
async fn main() {
    // Load environment variables from .env file if it exists
    dotenvy::dotenv().ok();

    init_tracing();

    let config = match AppConfig::from_env() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Failed to load configuration: {}. Exiting.", e);
            std::process::exit(1);
        }
    };

    if config.github_token.is_none() {
        tracing::warn!("Running without GITHUB_TOKEN. Rate limits will be strict.");
    }

    let github_client = match GitHubClient::new(config.github_token.clone()) {
        Ok(client) => client,
        Err(e) => {
            tracing::error!("Failed to initialize GitHub client: {}. Exiting.", e);
            std::process::exit(1);
        }
    };

    let metrics_cache = MetricsCache::new(&config, github_client.clone());

    let state = Arc::new(AppState {
        github_client,
        metrics_cache,
        config,
    });

    let state_clone = state.clone();
    tokio::spawn(async move {
        preload_popular_repos(state_clone).await;
    });

    let serve_dir = ServeDir::new("dist").not_found_service(ServeFile::new("dist/index.html"));

    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/repos/popular", get(get_popular_repos))
        .route("/api/repos/{owner}/{repo}/metrics", get(get_repo_metrics))
        .fallback_service(serve_dir)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = get_listener().await;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("failed to start server");
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

async fn get_listener() -> tokio::net::TcpListener {
    let port_str = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let port = match port_str.parse::<u16>() {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Invalid PORT value '{}': {}. Exiting.", port_str, e);
            std::process::exit(1);
        }
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Server listening on {}", addr);

    tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind TCP listener")
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "repoflow-backend",
        version: env!("CARGO_PKG_VERSION"),
    })
}

async fn get_popular_repos(State(state): State<Arc<AppState>>) -> Json<Vec<RepoId>> {
    Json(state.config.popular_repos.clone())
}

async fn get_repo_metrics(
    Path(repo_id): Path<RepoId>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<metrics::RepoMetricsResponse>, (axum::http::StatusCode, String)> {
    if let Some(cached_metrics) = state.metrics_cache.get(&repo_id).await {
        tracing::debug!(repo_id = %repo_id, "Returning cached metrics");
        return Ok(Json(cached_metrics));
    }

    match state.github_client.fetch_and_calculate_metrics(&state.config, &repo_id).await {
        Ok(metrics) => {
            state.metrics_cache.insert(repo_id, metrics.clone()).await;
            Ok(Json(metrics))
        }
        Err(e) => {
            tracing::error!("Failed to fetch PRs for {}: {}", repo_id, e);

            if let Some(octocrab::Error::GitHub { source, .. }) =
                e.downcast_ref::<octocrab::Error>()
            {
                // TODO(#29): Refactor this brittle string matching.
                // We should inspect the raw HTTP status code or use a strongly-typed error variant if available.
                if source.message.to_lowercase().contains("rate limit") {
                    return Err((
                        axum::http::StatusCode::TOO_MANY_REQUESTS,
                        "GitHub Rate Limit Exceeded".to_string(),
                    ));
                }
                if source.message.to_lowercase().contains("not found") {
                    return Err((
                        axum::http::StatusCode::NOT_FOUND,
                        "Repository Not Found".to_string(),
                    ));
                }
            }

            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_string(),
            ))
        }
    }
}

async fn preload_popular_repos(state: Arc<AppState>) {
    tracing::info!(
        "Preloading {} popular repositories",
        state.config.popular_repos.len()
    );

    stream::iter(&state.config.popular_repos)
        .for_each_concurrent(state.config.max_concurrent_preloads, |repo| {
            let state = state.clone();
            async move {
                preload_single_repo(&state, repo.clone()).await;
            }
        })
        .await;

    tracing::info!("Finished preloading popular repositories");
}

async fn preload_single_repo(state: &AppState, repo_id: RepoId) {
    // Check if already cached (unlikely during startup but good practice)
    if state.metrics_cache.get(&repo_id).await.is_some() {
        return;
    }

    match state.github_client.fetch_and_calculate_metrics(&state.config, &repo_id).await {
        Ok(metrics) => {
            state
                .metrics_cache
                .insert(repo_id.clone(), metrics)
                .await;
            tracing::info!(repo_id = %repo_id, "Preloaded metrics");
        }
        Err(e) => {
            tracing::warn!(repo_id = %repo_id, error = %e, "Failed to preload metrics");
        }
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if tokio::signal::ctrl_c().await.is_err() {
            tracing::error!(
                "Failed to install Ctrl+C handler. Graceful shutdown on Ctrl+C will not work."
            );
            std::future::pending::<()>().await;
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut stream) => {
                stream.recv().await;
            }
            Err(e) => {
                tracing::error!(
                    "Failed to install SIGTERM handler: {}. Graceful shutdown on SIGTERM will not work.",
                    e
                );
                std::future::pending::<()>().await;
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("signal received, starting graceful shutdown");
}
