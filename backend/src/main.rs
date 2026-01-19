use axum::{
    routing::get,
    Router,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Initialize tracing (logging)
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Define routes
    let app = Router::new()
        .route("/health", get(health_check));

    // Address to bind to
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Server listening on {}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "service": "repoflow-backend",
        "version": env!("CARGO_PKG_VERSION")
    }))
}