use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::{auth, models::*};

// ── Error helper ──────────────────────────────────────────────────────────────

pub struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": self.0.to_string()})),
        )
            .into_response()
    }
}

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(e: E) -> Self {
        AppError(e.into())
    }
}

type Result<T> = std::result::Result<T, AppError>;

// ── Token extraction ──────────────────────────────────────────────────────────

fn user_id_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .and_then(|token| auth::verify_token(token).ok())
        .map(|claims| claims.sub)
}

// ── Auth handlers ─────────────────────────────────────────────────────────────

pub async fn register(
    State(pool): State<SqlitePool>,
    Json(req): Json<RegisterRequest>,
) -> Result<Response> {
    let existing = sqlx::query("SELECT id FROM users WHERE email = ?")
        .bind(&req.email)
        .fetch_optional(&pool)
        .await?;

    if existing.is_some() {
        return Ok((
            StatusCode::CONFLICT,
            Json(json!({"error": "Email already registered"})),
        )
            .into_response());
    }

    let id = Uuid::new_v4().to_string();
    let hash = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES (?, ?, ?)")
        .bind(&id)
        .bind(&req.email)
        .bind(&hash)
        .execute(&pool)
        .await?;

    sqlx::query("INSERT INTO user_state (user_id) VALUES (?)")
        .bind(&id)
        .execute(&pool)
        .await?;

    let token = auth::create_token(&id)?;
    Ok((StatusCode::CREATED, Json(json!({"token": token}))).into_response())
}

pub async fn login(
    State(pool): State<SqlitePool>,
    Json(req): Json<LoginRequest>,
) -> Result<Response> {
    let row = sqlx::query("SELECT id, password_hash FROM users WHERE email = ?")
        .bind(&req.email)
        .fetch_optional(&pool)
        .await?;

    let Some(row) = row else {
        return Ok((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid credentials"})),
        )
            .into_response());
    };

    let id: String = row.get("id");
    let hash: String = row.get("password_hash");

    let valid = bcrypt::verify(&req.password, &hash)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    if !valid {
        return Ok((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid credentials"})),
        )
            .into_response());
    }

    let token = auth::create_token(&id)?;
    Ok((StatusCode::OK, Json(json!({"token": token}))).into_response())
}

// ── State handlers ────────────────────────────────────────────────────────────

pub async fn get_state(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
) -> Result<Response> {
    let Some(user_id) = user_id_from_headers(&headers) else {
        return Ok((StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response());
    };

    let row = sqlx::query("SELECT state_json FROM user_state WHERE user_id = ?")
        .bind(&user_id)
        .fetch_optional(&pool)
        .await?;

    let state: Value = row
        .and_then(|r| {
            let s: String = r.get("state_json");
            serde_json::from_str(&s).ok()
        })
        .unwrap_or_else(|| json!({"configs":[],"items":[],"groups":[],"timeCosts":[]}));

    Ok((StatusCode::OK, Json(state)).into_response())
}

pub async fn put_state(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    Json(state): Json<Value>,
) -> Result<Response> {
    let Some(user_id) = user_id_from_headers(&headers) else {
        return Ok((StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response());
    };

    let state_json = serde_json::to_string(&state)?;

    sqlx::query(
        "INSERT INTO user_state (user_id, state_json) VALUES (?, ?)
         ON CONFLICT(user_id) DO UPDATE SET state_json = excluded.state_json",
    )
    .bind(&user_id)
    .bind(&state_json)
    .execute(&pool)
    .await?;

    Ok((StatusCode::OK, Json(json!({"ok": true}))).into_response())
}

// ── Static file fallback ──────────────────────────────────────────────────────

pub async fn serve_index() -> impl IntoResponse {
    match tokio::fs::read_to_string("../index.html").await {
        Ok(html) => Html(html).into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            "index.html not found — run `cargo run` from backend-rs/",
        )
            .into_response(),
    }
}
