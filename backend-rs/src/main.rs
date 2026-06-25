use axum::{
    routing::{delete, get, patch, post, put},
    Router,
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use std::str::FromStr;
use tower_http::cors::CorsLayer;

mod auth;
mod cost;
mod handlers;
mod models;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:bijoux.db".to_string());

    let opts = SqliteConnectOptions::from_str(&db_url)?
        .create_if_missing(true)
        .foreign_keys(true);

    let pool = SqlitePool::connect_with(opts).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    let api = Router::new()
        .route("/auth/register", post(handlers::register))
        .route("/auth/login", post(handlers::login))
        .route("/state", get(handlers::get_state).delete(handlers::delete_state))
        .route("/configs", post(handlers::create_config))
        .route("/configs/:id", put(handlers::update_config).delete(handlers::delete_config))
        .route("/configs/:id/supplies", post(handlers::create_supply))
        .route("/supplies/:id", delete(handlers::delete_supply))
        .route("/time-costs", post(handlers::create_time_cost))
        .route("/time-costs/:id", put(handlers::update_time_cost).delete(handlers::delete_time_cost))
        .route("/groups", post(handlers::create_group))
        .route("/groups/:id", put(handlers::update_group).delete(handlers::delete_group))
        .route("/items", post(handlers::create_item))
        .route("/items/:id", put(handlers::update_item).delete(handlers::delete_item))
        .route("/items/:id/position", patch(handlers::patch_item_position));

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
