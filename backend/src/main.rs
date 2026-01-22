mod cache;
mod github;
mod metrics;

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use cache::MetricsCache;
use futures::stream::{self, StreamExt};
use github::GitHubClient;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Maximum number of concurrent requests for preloading popular repositories.
const MAX_CONCURRENT_PRELOADS: usize = 4;

/// List of popular repositories to preload and display in the UI.
const POPULAR_REPOS: &[(&str, &str)] = &[
    ("facebook", "react"),
    ("rust-lang", "rust"),
    ("vercel", "next.js"),
    ("tailwindlabs", "tailwindcss"),
    ("microsoft", "vscode"),
    ("rust-lang", "rust-analyzer"),
];

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    version: &'static str,
}

#[derive(Serialize, Clone)]
struct PopularRepo {
    owner: String,
    repo: String,
}

/// Shared application state accessible to all request handlers.
struct AppState {
    /// Thread-safe client for interacting with the GitHub API.
    github_client: GitHubClient,
    /// Encapsulated cache for repository metrics with automatic background refresh.
    metrics_cache: MetricsCache,
}

/// Parameters extracted from the URL path /api/repos/:owner/:repo/metrics
#[derive(Deserialize)]
struct RepoPath {
    owner: String,
    repo: String,
}

#[tokio::main]
async fn main() {
    init_tracing();

    let github_token = std::env::var("GITHUB_TOKEN").ok();
    let github_client = match GitHubClient::new(github_token) {
        Ok(client) => client,
        Err(e) => {
            tracing::error!("Failed to initialize GitHub client: {}. Exiting.", e);
            std::process::exit(1);
        }
    };

    let metrics_cache = MetricsCache::new(github_client.clone());

    let state = Arc::new(AppState {
        github_client,
        metrics_cache,
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

async fn get_popular_repos() -> Json<Vec<PopularRepo>> {
    let repos = POPULAR_REPOS
        .iter()
        .map(|(owner, repo)| PopularRepo {
            owner: owner.to_string(),
            repo: repo.to_string(),
        })
        .collect();
    Json(repos)
}

async fn get_repo_metrics(
    Path(RepoPath { owner, repo }): Path<RepoPath>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<metrics::RepoMetricsResponse>, (axum::http::StatusCode, String)> {
    let metrics = state
        .metrics_cache
        .get(&state.github_client, &owner, &repo)
        .await?;

    Ok(Json(metrics))
}

async fn preload_popular_repos(state: Arc<AppState>) {
    tracing::info!("Preloading {} popular repositories", POPULAR_REPOS.len());

    stream::iter(POPULAR_REPOS)
        .for_each_concurrent(MAX_CONCURRENT_PRELOADS, |&(owner, repo)| {
            let state = state.clone();
            async move {
                preload_single_repo(&state, owner, repo).await;
            }
        })
        .await;

    tracing::info!("Finished preloading popular repositories");
}

async fn preload_single_repo(state: &AppState, owner: &str, repo: &str) {
    match state
        .metrics_cache
        .get(&state.github_client, owner, repo)
        .await
    {
        Ok(_) => {
            tracing::info!(owner = %owner, repo = %repo, "Preloaded metrics");
        }
        Err((status, msg)) => {
            tracing::warn!(owner = %owner, repo = %repo, status = ?status, msg = %msg, "Failed to preload metrics");
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
