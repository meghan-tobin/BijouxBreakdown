use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

// ── Request bodies ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateConfigReq {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub locked: bool,
    pub color_idx: Option<i64>,
    pub chip_salt: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateConfigReq {
    pub name: String,
    pub locked: bool,
    pub color_idx: Option<i64>,
    pub chip_salt: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSupplyReq {
    pub id: String,
    pub name: String,
    pub cost: f64,
    pub quantity: f64,
    pub unit: String,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ItemUsageReq {
    pub supply_id: String,
    pub amount: f64,
    pub unit: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTimeCostReq {
    pub id: String,
    pub name: String,
    pub duration: f64,
    pub duration_unit: String,
    pub rate: f64,
    pub color_idx: Option<i64>,
    pub config_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTimeCostReq {
    pub name: String,
    pub duration: f64,
    pub duration_unit: String,
    pub rate: f64,
    pub color_idx: Option<i64>,
    pub config_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGroupReq {
    pub id: String,
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub color_idx: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGroupReq {
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub color_idx: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateItemReq {
    pub id: String,
    pub name: String,
    pub config_id: Option<String>,
    pub usages: Vec<ItemUsageReq>,
    pub time_cost_id: Option<String>,
    pub time_amount: Option<f64>,
    pub time_unit: Option<String>,
    pub price: Option<f64>,
    pub group_id: Option<String>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub rx: Option<f64>,
    pub ry: Option<f64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateItemReq {
    pub name: String,
    pub config_id: Option<String>,
    pub usages: Vec<ItemUsageReq>,
    pub time_cost_id: Option<String>,
    pub time_amount: Option<f64>,
    pub time_unit: Option<String>,
    pub price: Option<f64>,
    pub group_id: Option<String>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub rx: Option<f64>,
    pub ry: Option<f64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchItemPosition {
    #[serde(default)]
    pub x: Option<f64>,
    #[serde(default)]
    pub y: Option<f64>,
    #[serde(default)]
    pub group_id: Option<String>,
    #[serde(default)]
    pub rx: Option<f64>,
    #[serde(default)]
    pub ry: Option<f64>,
    #[serde(default)]
    pub width: Option<f64>,
    #[serde(default)]
    pub height: Option<f64>,
}
