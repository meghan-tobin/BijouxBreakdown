use axum::{
    routing::{get, post},
    Router,
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use std::str::FromStr;
use tower_http::cors::CorsLayer;

mod auth;
mod handlers;
mod models;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:bijoux.db".to_string());

    let opts = SqliteConnectOptions::from_str(&db_url)?.create_if_missing(true);
    let pool = SqlitePool::connect_with(opts).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    let api = Router::new()
        .route("/auth/register", post(handlers::register))
        .route("/auth/login", post(handlers::login))
        .route("/state", get(handlers::get_state).put(handlers::put_state));

    let app = Router::new()
        .nest("/api", api)
        .fallback(handlers::serve_index)
        .layer(CorsLayer::permissive())
        .with_state(pool);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    println!("BijouxBreakdown running at http://localhost:{}", port);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
