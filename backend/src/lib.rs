pub mod config;
pub mod metrics;
pub mod querier;

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use config::{AppConfig, RepoId};
use querier::MetricsQuerier;
use serde::Serialize;
use std::sync::Arc;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
    service: &'static str,
    version: &'static str,
}

/// Shared application state accessible to all request handlers.
pub struct AppState {
    /// Service for querying repository metrics.
    pub querier: MetricsQuerier,
    /// Application configuration loaded from environment variables.
    pub config: AppConfig,
}

impl AppState {
    /// Initializes the application state, including the metrics querier.
    pub fn new(config: AppConfig) -> anyhow::Result<Self> {
        let querier = MetricsQuerier::new(&config)?;
        Ok(Self { querier, config })
    }
}

pub fn create_app(state: Arc<AppState>) -> Router {
    let serve_dir = ServeDir::new("dist").not_found_service(ServeFile::new("dist/index.html"));

    Router::new()
        .route("/api/health", get(health_check))
        .route("/api/repos/popular", get(get_popular_repos))
        .route("/api/repos/{owner}/{repo}/metrics", get(get_repo_metrics))
        .fallback_service(serve_dir)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "repoflow-backend",
        version: env!("CARGO_PKG_VERSION"),
    })
}

pub async fn get_popular_repos(State(state): State<Arc<AppState>>) -> Json<Vec<RepoId>> {
    Json(state.config.popular_repos.clone())
}

pub async fn get_repo_metrics(
    Path(repo_id): Path<RepoId>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<metrics::RepoMetricsResponse>, (axum::http::StatusCode, String)> {
    match state.querier.get(repo_id.clone()).await {
        Ok(metrics) => {
            tracing::debug!(repo_id = %repo_id, "Returning metrics");
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
