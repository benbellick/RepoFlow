use axum::serve;
use backend::config::AppConfig;
use backend::{create_app, AppState};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

    let state = match AppState::new(config) {
        Ok(s) => Arc::new(s),
        Err(e) => {
            tracing::error!("Failed to initialize application state: {}. Exiting.", e);
            std::process::exit(1);
        }
    };

    let app = create_app(state);

    let listener = get_listener().await;

    serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("failed to start server");
}

fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "backend=debug,tower_http=debug".into());

    let log_format = std::env::var("LOG_FORMAT").unwrap_or_default();

    if log_format == "json" {
        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer().json().with_ansi(false))
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer())
            .init();
    }
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