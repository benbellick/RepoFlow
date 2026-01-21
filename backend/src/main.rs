mod github;
mod metrics;

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use chrono::{Duration, Utc};
use github::GitHubClient;
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Number of past days to fetch pull request data for from the GitHub API.
const PR_FETCH_DAYS: i64 = 90;
/// Hard limit on the number of paginated requests to make to the GitHub API per repository.
const MAX_GITHUB_API_PAGES: u32 = 10;
/// The number of individual data points (days) to return in the flow metrics response.
const METRICS_DAYS_TO_DISPLAY: i64 = 30;
/// The size of the trailing window (in days) used to calculate the rolling counts.
const METRICS_WINDOW_SIZE: i64 = 30;
/// Time to live for cached repository metrics (24 hours).
/// Note: This long TTL reduces GitHub API load but may result in stale data.
/// TODO(#15): Implement a more sophisticated cache invalidation or background refresh strategy.
const CACHE_TTL: StdDuration = StdDuration::from_secs(86400);
/// Maximum number of entries to keep in the metrics cache.
const CACHE_MAX_CAPACITY: u64 = 1000;

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
    /// In-memory cache for repository metrics to avoid redundant API calls and processing.
    metrics_cache: Cache<String, metrics::RepoMetricsResponse>,
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

    let metrics_cache = Cache::builder()
        .max_capacity(CACHE_MAX_CAPACITY)
        .time_to_live(CACHE_TTL)
        .build();

    let state = Arc::new(AppState {
        github_client,
        metrics_cache,
    });

    // Preload popular repos in the background
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
    let cache_key = get_cache_key(&owner, &repo);

    if let Some(cached_metrics) = state.metrics_cache.get(&cache_key).await {
        tracing::debug!(owner = %owner, repo = %repo, "Returning cached metrics");
        return Ok(Json(cached_metrics));
    }

    let metrics = fetch_and_calculate_metrics(&state, &owner, &repo).await?;

    state.metrics_cache.insert(cache_key, metrics.clone()).await;

    Ok(Json(metrics))
}

async fn fetch_and_calculate_metrics(
    state: &AppState,
    owner: &str,
    repo: &str,
) -> Result<metrics::RepoMetricsResponse, (axum::http::StatusCode, String)> {
    let prs = state
        .github_client
        .fetch_pull_requests(owner, repo, PR_FETCH_DAYS, MAX_GITHUB_API_PAGES)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch PRs for {}/{}: {}", owner, repo, e);

            if let Some(octocrab::Error::GitHub { source, .. }) =
                e.downcast_ref::<octocrab::Error>()
            {
                // TODO(#29): Refactor this brittle string matching.
                // We should inspect the raw HTTP status code or use a strongly-typed error variant if available.
                if source.message.to_lowercase().contains("rate limit") {
                    return (
                        axum::http::StatusCode::TOO_MANY_REQUESTS,
                        "GitHub Rate Limit Exceeded".to_string(),
                    );
                }
                if source.message.to_lowercase().contains("not found") {
                    return (
                        axum::http::StatusCode::NOT_FOUND,
                        "Repository Not Found".to_string(),
                    );
                }
            }

            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_string(),
            )
        })?;

    let metrics = metrics::calculate_metrics(
        &prs,
        Duration::days(METRICS_DAYS_TO_DISPLAY),
        Duration::days(METRICS_WINDOW_SIZE),
        Utc::now(),
    );

    Ok(metrics)
}

async fn preload_popular_repos(state: Arc<AppState>) {
    tracing::info!("Preloading {} popular repositories", POPULAR_REPOS.len());
    for (owner, repo) in POPULAR_REPOS {
        let cache_key = get_cache_key(owner, repo);
        match fetch_and_calculate_metrics(&state, owner, repo).await {
            Ok(metrics) => {
                state.metrics_cache.insert(cache_key, metrics).await;
                tracing::info!(owner = %owner, repo = %repo, "Preloaded metrics");
            }
            Err((status, msg)) => {
                tracing::warn!(owner = %owner, repo = %repo, status = ?status, msg = %msg, "Failed to preload metrics");
            }
        }
    }
    tracing::info!("Finished preloading popular repositories");
}
fn get_cache_key(owner: &str, repo: &str) -> String {
    format!("owner::{}/repo::{}", owner, repo)
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
