use axum::{
    extract::{Path, State},
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
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": self.0.to_string()}))).into_response()
    }
}

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(e: E) -> Self { AppError(e.into()) }
}

type Result<T> = std::result::Result<T, AppError>;

// ── Auth helpers ──────────────────────────────────────────────────────────────

fn user_id_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .and_then(|token| auth::verify_token(token).ok())
        .map(|claims| claims.sub)
}

macro_rules! require_auth {
    ($headers:expr) => {
        match user_id_from_headers(&$headers) {
            Some(id) => id,
            None => return Ok((StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response()),
        }
    };
}

fn ok() -> Response {
    (StatusCode::OK, Json(json!({"ok": true}))).into_response()
}

fn created() -> Response {
    (StatusCode::CREATED, Json(json!({"ok": true}))).into_response()
}

fn new_id() -> String {
    Uuid::new_v4().to_string()
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
        return Ok((StatusCode::CONFLICT, Json(json!({"error": "Email already registered"}))).into_response());
    }

    let id = new_id();
    let hash = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES (?, ?, ?)")
        .bind(&id).bind(&req.email).bind(&hash)
        .execute(&pool).await?;

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
        return Ok((StatusCode::UNAUTHORIZED, Json(json!({"error": "Invalid credentials"}))).into_response());
    };

    let id: String = row.get("id");
    let hash: String = row.get("password_hash");

    let valid = bcrypt::verify(&req.password, &hash)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    if !valid {
        return Ok((StatusCode::UNAUTHORIZED, Json(json!({"error": "Invalid credentials"}))).into_response());
    }

    let token = auth::create_token(&id)?;
    Ok((StatusCode::OK, Json(json!({"token": token}))).into_response())
}

// ── State: read all ───────────────────────────────────────────────────────────

pub async fn get_state(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
) -> Result<Response> {
    let user_id = require_auth!(headers);

    let config_rows = sqlx::query(
        "SELECT id, name, locked, color_idx, chip_salt FROM configs WHERE user_id = ? ORDER BY sort_order"
    ).bind(&user_id).fetch_all(&pool).await?;

    let supply_rows = sqlx::query(
        "SELECT s.id, s.config_id, s.name, s.cost, s.quantity, s.unit
         FROM supplies s JOIN configs c ON c.id = s.config_id WHERE c.user_id = ?"
    ).bind(&user_id).fetch_all(&pool).await?;

    let tc_rows = sqlx::query(
        "SELECT id, config_id, name, duration, duration_unit, rate, color_idx
         FROM time_costs WHERE user_id = ?"
    ).bind(&user_id).fetch_all(&pool).await?;

    let group_rows = sqlx::query(
        "SELECT id, name, x, y, width, height, color_idx FROM groups WHERE user_id = ?"
    ).bind(&user_id).fetch_all(&pool).await?;

    let item_rows = sqlx::query(
        "SELECT id, config_id, name, price, time_cost_id, time_amount, time_unit,
                x, y, width, height, group_id, rx, ry
         FROM items WHERE user_id = ?"
    ).bind(&user_id).fetch_all(&pool).await?;

    let usage_rows = sqlx::query(
        "SELECT u.item_id, u.supply_id, u.amount, u.unit
         FROM item_usages u JOIN items i ON i.id = u.item_id WHERE i.user_id = ?"
    ).bind(&user_id).fetch_all(&pool).await?;

    let configs: Vec<Value> = config_rows.iter().map(|r| {
        let config_id: String = r.get("id");
        let supplies: Vec<Value> = supply_rows.iter()
            .filter(|s| s.get::<String, _>("config_id") == config_id)
            .map(|s| json!({
                "id":       s.get::<String, _>("id"),
                "name":     s.get::<String, _>("name"),
                "cost":     s.get::<f64, _>("cost"),
                "quantity": s.get::<f64, _>("quantity"),
                "unit":     s.get::<String, _>("unit"),
            }))
            .collect();
        json!({
            "id":       config_id,
            "name":     r.get::<String, _>("name"),
            "locked":   r.get::<i64, _>("locked") != 0,
            "colorIdx": r.get::<Option<i64>, _>("color_idx"),
            "chipSalt": r.get::<Option<String>, _>("chip_salt"),
            "supplies": supplies,
        })
    }).collect();

    let time_costs: Vec<Value> = tc_rows.iter().map(|r| json!({
        "id":           r.get::<String, _>("id"),
        "name":         r.get::<String, _>("name"),
        "duration":     r.get::<f64, _>("duration"),
        "durationUnit": r.get::<String, _>("duration_unit"),
        "rate":         r.get::<f64, _>("rate"),
        "colorIdx":     r.get::<Option<i64>, _>("color_idx"),
        "configId":     r.get::<Option<String>, _>("config_id"),
    })).collect();

    let groups: Vec<Value> = group_rows.iter().map(|r| json!({
        "id":       r.get::<String, _>("id"),
        "name":     r.get::<String, _>("name"),
        "x":        r.get::<f64, _>("x"),
        "y":        r.get::<f64, _>("y"),
        "width":    r.get::<f64, _>("width"),
        "height":   r.get::<f64, _>("height"),
        "colorIdx": r.get::<Option<i64>, _>("color_idx"),
    })).collect();

    let items: Vec<Value> = item_rows.iter().map(|r| {
        let item_id: String = r.get("id");
        let usages: Vec<Value> = usage_rows.iter()
            .filter(|u| u.get::<String, _>("item_id") == item_id)
            .map(|u| json!({
                "supplyId": u.get::<String, _>("supply_id"),
                "amount":   u.get::<f64, _>("amount"),
                "unit":     u.get::<Option<String>, _>("unit"),
            }))
            .collect();
        json!({
            "id":         item_id,
            "name":       r.get::<String, _>("name"),
            "configId":   r.get::<Option<String>, _>("config_id"),
            "usages":     usages,
            "timeCostId": r.get::<Option<String>, _>("time_cost_id"),
            "timeAmount": r.get::<Option<f64>, _>("time_amount"),
            "timeUnit":   r.get::<Option<String>, _>("time_unit"),
            "price":      r.get::<Option<f64>, _>("price"),
            "groupId":    r.get::<Option<String>, _>("group_id"),
            "x":          r.get::<Option<f64>, _>("x"),
            "y":          r.get::<Option<f64>, _>("y"),
            "width":      r.get::<Option<f64>, _>("width"),
            "height":     r.get::<Option<f64>, _>("height"),
            "rx":         r.get::<Option<f64>, _>("rx"),
            "ry":         r.get::<Option<f64>, _>("ry"),
        })
    }).collect();

    Ok((StatusCode::OK, Json(json!({
        "configs":   configs,
        "items":     items,
        "groups":    groups,
        "timeCosts": time_costs,
    }))).into_response())
}

// ── State: reset all ──────────────────────────────────────────────────────────

pub async fn delete_state(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
) -> Result<Response> {
    let user_id = require_auth!(headers);
    sqlx::query("DELETE FROM items WHERE user_id = ?").bind(&user_id).execute(&pool).await?;
    sqlx::query("DELETE FROM groups WHERE user_id = ?").bind(&user_id).execute(&pool).await?;
    sqlx::query("DELETE FROM time_costs WHERE user_id = ?").bind(&user_id).execute(&pool).await?;
    sqlx::query("DELETE FROM configs WHERE user_id = ?").bind(&user_id).execute(&pool).await?;
    Ok(ok())
}

// ── Configs ───────────────────────────────────────────────────────────────────

pub async fn create_config(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    Json(req): Json<CreateConfigReq>,
) -> Result<Response> {
    let user_id = require_auth!(headers);

    let max: i64 = sqlx::query("SELECT COALESCE(MAX(sort_order), -1) FROM configs WHERE user_id = ?")
        .bind(&user_id).fetch_one(&pool).await?.get(0);

    sqlx::query(
        "INSERT INTO configs (id, user_id, name, locked, color_idx, chip_salt, sort_order)
         VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&req.id).bind(&user_id).bind(&req.name)
    .bind(req.locked as i64).bind(req.color_idx).bind(&req.chip_salt)
    .bind(max + 1)
    .execute(&pool).await?;

    Ok(created())
}

pub async fn update_config(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(req): Json<UpdateConfigReq>,
) -> Result<Response> {
    let user_id = require_auth!(headers);
    sqlx::query(
        "UPDATE configs SET name = ?, locked = ?, color_idx = ?, chip_salt = ?
         WHERE id = ? AND user_id = ?"
    )
    .bind(&req.name).bind(req.locked as i64).bind(req.color_idx).bind(&req.chip_salt)
    .bind(&id).bind(&user_id)
    .execute(&pool).await?;
    Ok(ok())
}

pub async fn delete_config(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Response> {
    let user_id = require_auth!(headers);
    sqlx::query("DELETE FROM configs WHERE id = ? AND user_id = ?")
        .bind(&id).bind(&user_id).execute(&pool).await?;
    Ok(ok())
}

// ── Supplies ──────────────────────────────────────────────────────────────────

pub async fn create_supply(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    Path(config_id): Path<String>,
    Json(req): Json<CreateSupplyReq>,
) -> Result<Response> {
    let user_id = require_auth!(headers);

    let cfg = sqlx::query("SELECT id FROM configs WHERE id = ? AND user_id = ?")
        .bind(&config_id).bind(&user_id).fetch_optional(&pool).await?;
    if cfg.is_none() {
        return Ok((StatusCode::NOT_FOUND, Json(json!({"error": "Config not found"}))).into_response());
    }

    sqlx::query(
        "INSERT INTO supplies (id, config_id, name, cost, quantity, unit) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&req.id).bind(&config_id).bind(&req.name).bind(req.cost).bind(req.quantity).bind(&req.unit)
    .execute(&pool).await?;

    Ok(created())
}

pub async fn delete_supply(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    Path(supply_id): Path<String>,
) -> Result<Response> {
    let user_id = require_auth!(headers);
    sqlx::query(
        "DELETE FROM supplies WHERE id = ?
         AND config_id IN (SELECT id FROM configs WHERE user_id = ?)"
    )
    .bind(&supply_id).bind(&user_id).execute(&pool).await?;
    Ok(ok())
}

// ── Time costs ────────────────────────────────────────────────────────────────

pub async fn create_time_cost(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    Json(req): Json<CreateTimeCostReq>,
) -> Result<Response> {
    let user_id = require_auth!(headers);
    sqlx::query(
        "INSERT INTO time_costs (id, user_id, config_id, name, duration, duration_unit, rate, color_idx)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&req.id).bind(&user_id).bind(&req.config_id).bind(&req.name)
    .bind(req.duration).bind(&req.duration_unit).bind(req.rate).bind(req.color_idx)
    .execute(&pool).await?;
    Ok(created())
}

pub async fn update_time_cost(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(req): Json<UpdateTimeCostReq>,
) -> Result<Response> {
    let user_id = require_auth!(headers);
    sqlx::query(
        "UPDATE time_costs SET name = ?, duration = ?, duration_unit = ?, rate = ?,
         color_idx = ?, config_id = ? WHERE id = ? AND user_id = ?"
    )
    .bind(&req.name).bind(req.duration).bind(&req.duration_unit).bind(req.rate)
    .bind(req.color_idx).bind(&req.config_id).bind(&id).bind(&user_id)
    .execute(&pool).await?;
    Ok(ok())
}

pub async fn delete_time_cost(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Response> {
    let user_id = require_auth!(headers);
    sqlx::query("DELETE FROM time_costs WHERE id = ? AND user_id = ?")
        .bind(&id).bind(&user_id).execute(&pool).await?;
    Ok(ok())
}

// ── Groups ────────────────────────────────────────────────────────────────────

pub async fn create_group(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    Json(req): Json<CreateGroupReq>,
) -> Result<Response> {
    let user_id = require_auth!(headers);
    sqlx::query(
        "INSERT INTO groups (id, user_id, name, x, y, width, height, color_idx) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&req.id).bind(&user_id).bind(&req.name)
    .bind(req.x).bind(req.y).bind(req.width).bind(req.height).bind(req.color_idx)
    .execute(&pool).await?;
    Ok(created())
}

pub async fn update_group(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(req): Json<UpdateGroupReq>,
) -> Result<Response> {
    let user_id = require_auth!(headers);
    sqlx::query(
        "UPDATE groups SET name = ?, x = ?, y = ?, width = ?, height = ?, color_idx = ?
         WHERE id = ? AND user_id = ?"
    )
    .bind(&req.name).bind(req.x).bind(req.y).bind(req.width).bind(req.height).bind(req.color_idx)
    .bind(&id).bind(&user_id)
    .execute(&pool).await?;
    Ok(ok())
}

pub async fn delete_group(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Response> {
    let user_id = require_auth!(headers);
    sqlx::query("DELETE FROM groups WHERE id = ? AND user_id = ?")
        .bind(&id).bind(&user_id).execute(&pool).await?;
    Ok(ok())
}

// ── Items ─────────────────────────────────────────────────────────────────────

pub async fn create_item(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    Json(req): Json<CreateItemReq>,
) -> Result<Response> {
    let user_id = require_auth!(headers);
    let mut tx = pool.begin().await?;

    sqlx::query(
        "INSERT INTO items (id, user_id, config_id, name, price, time_cost_id, time_amount,
         time_unit, x, y, group_id, rx, ry) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&req.id).bind(&user_id).bind(&req.config_id).bind(&req.name).bind(req.price)
    .bind(&req.time_cost_id).bind(req.time_amount).bind(&req.time_unit)
    .bind(req.x).bind(req.y).bind(&req.group_id).bind(req.rx).bind(req.ry)
    .execute(&mut *tx).await?;

    for u in &req.usages {
        sqlx::query(
            "INSERT INTO item_usages (id, item_id, supply_id, amount, unit) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&new_id()).bind(&req.id).bind(&u.supply_id).bind(u.amount).bind(&u.unit)
        .execute(&mut *tx).await?;
    }

    tx.commit().await?;
    Ok(created())
}

pub async fn update_item(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(req): Json<UpdateItemReq>,
) -> Result<Response> {
    let user_id = require_auth!(headers);

    let exists = sqlx::query("SELECT id FROM items WHERE id = ? AND user_id = ?")
        .bind(&id).bind(&user_id).fetch_optional(&pool).await?;
    if exists.is_none() {
        return Ok((StatusCode::NOT_FOUND, Json(json!({"error": "Item not found"}))).into_response());
    }

    let mut tx = pool.begin().await?;

    sqlx::query(
        "UPDATE items SET name = ?, config_id = ?, price = ?, time_cost_id = ?, time_amount = ?,
         time_unit = ?, group_id = ?, x = ?, y = ?, width = ?, height = ?, rx = ?, ry = ?
         WHERE id = ? AND user_id = ?"
    )
    .bind(&req.name).bind(&req.config_id).bind(req.price)
    .bind(&req.time_cost_id).bind(req.time_amount).bind(&req.time_unit)
    .bind(&req.group_id).bind(req.x).bind(req.y).bind(req.width).bind(req.height)
    .bind(req.rx).bind(req.ry).bind(&id).bind(&user_id)
    .execute(&mut *tx).await?;

    sqlx::query("DELETE FROM item_usages WHERE item_id = ?")
        .bind(&id).execute(&mut *tx).await?;

    for u in &req.usages {
        sqlx::query(
            "INSERT INTO item_usages (id, item_id, supply_id, amount, unit) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&new_id()).bind(&id).bind(&u.supply_id).bind(u.amount).bind(&u.unit)
        .execute(&mut *tx).await?;
    }

    tx.commit().await?;
    Ok(ok())
}

pub async fn patch_item_position(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(req): Json<PatchItemPosition>,
) -> Result<Response> {
    let user_id = require_auth!(headers);
    sqlx::query(
        "UPDATE items SET x = ?, y = ?, group_id = ?, rx = ?, ry = ?, width = ?, height = ?
         WHERE id = ? AND user_id = ?"
    )
    .bind(req.x).bind(req.y).bind(&req.group_id).bind(req.rx).bind(req.ry)
    .bind(req.width).bind(req.height).bind(&id).bind(&user_id)
    .execute(&pool).await?;
    Ok(ok())
}

pub async fn delete_item(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Response> {
    let user_id = require_auth!(headers);
    sqlx::query("DELETE FROM items WHERE id = ? AND user_id = ?")
        .bind(&id).bind(&user_id).execute(&pool).await?;
    Ok(ok())
}

// ── Static file fallback ──────────────────────────────────────────────────────

pub async fn serve_index() -> impl IntoResponse {
    match tokio::fs::read_to_string("../index.html").await {
        Ok(html) => Html(html).into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            "index.html not found — run `cargo run` from backend-rs/",
        ).into_response(),
    }
}
