mod github;
mod metrics;

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use github::GitHubClient;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::services::{ServeDir, ServeFile};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const PR_FETCH_DAYS: i64 = 90;
const MAX_GITHUB_API_PAGES: u32 = 10;
const METRICS_DAYS_TO_DISPLAY: i64 = 30;
const METRICS_WINDOW_SIZE: i64 = 30;

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

    let state = Arc::new(AppState { github_client });

    let serve_dir = ServeDir::new("dist").not_found_service(ServeFile::new("dist/index.html"));

    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/repos/:owner/:repo/metrics", get(get_repo_metrics))
        .fallback_service(serve_dir)
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

async fn get_repo_metrics(
    Path(RepoPath { owner, repo }): Path<RepoPath>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<metrics::FlowMetricsResponse>>, (axum::http::StatusCode, String)> {
    let prs = state
        .github_client
        .fetch_pull_requests(&owner, &repo, PR_FETCH_DAYS, MAX_GITHUB_API_PAGES)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch PRs for {}/{}: {}", owner, repo, e);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_string(),
            )
        })?;

    let metrics = metrics::calculate_metrics(&prs, METRICS_DAYS_TO_DISPLAY, METRICS_WINDOW_SIZE);

    Ok(Json(metrics))
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
