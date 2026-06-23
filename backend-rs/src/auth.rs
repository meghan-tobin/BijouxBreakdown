use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::Claims;

fn secret() -> Vec<u8> {
    std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "bijoux-dev-secret-change-in-production".to_string())
        .into_bytes()
}

pub fn create_token(user_id: &str) -> anyhow::Result<String> {
    let exp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
        + 30 * 24 * 3600; // 30 days

    let claims = Claims {
        sub: user_id.to_string(),
        exp,
    };

    Ok(encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(&secret()),
    )?)
}

pub fn verify_token(token: &str) -> anyhow::Result<Claims> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(&secret()),
        &Validation::default(),
    )?;
    Ok(data.claims)
}
