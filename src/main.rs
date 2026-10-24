use std::{env, net::SocketAddr, sync::Arc};

use axum_experiment::{app, infrastructure::postgres::PgUserRepository};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .json()
        .init();

    let app_env = env::var("APP_ENV").unwrap_or_else(|_| "development".to_owned());
    let database_url = match env::var("DATABASE_URL") {
        Ok(database_url) => database_url,
        Err(_) if app_env != "production" => {
            "postgres://app:localdev@localhost:5432/axum_experiment".to_owned()
        }
        Err(_) => return Err("DATABASE_URL must be set in production".into()),
    };
    let max_connections = env::var("DATABASE_MAX_CONNECTIONS")
        .unwrap_or_else(|_| "10".to_owned())
        .parse::<u32>()?;
    if max_connections == 0 {
        return Err("DATABASE_MAX_CONNECTIONS must be greater than zero".into());
    }
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(&database_url)
        .await?;

    if app_env != "production" || env::var("RUN_MIGRATIONS").as_deref() == Ok("true") {
        migrate(&pool).await?;
    }

    let repository = Arc::new(PgUserRepository { pool });
    let application = app(repository);
    let host = env::var("HTTP_HOST").unwrap_or_else(|_| "127.0.0.1".to_owned());
    let port = env::var("HTTP_PORT")
        .unwrap_or_else(|_| "3000".to_owned())
        .parse::<u16>()?;
    let address = SocketAddr::new(host.parse()?, port);
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!(%address, "HTTP server listening");
    axum::serve(listener, application)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn migrate(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for Ctrl+C")
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! { _ = ctrl_c => {}, _ = terminate => {} }
    tracing::info!("shutdown signal received");
}
