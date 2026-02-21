use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub base_currency: String,
}

#[derive(Deserialize, Debug)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Debug)]
pub struct AuthResponse {
    pub token: String,
    pub refresh_token: String,
    pub message: String,
}

#[derive(Deserialize, Debug)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Deserialize, Debug)]
pub struct OAuthLoginRequest {
    pub provider: String,
    pub id_token: String,
    pub username: Option<String>,
    pub base_currency: Option<String>,
}

#[derive(Serialize, Debug, sqlx::FromRow)]
pub struct RefreshTokenRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: Option<DateTime<Utc>>,
    pub replaced_by: Option<String>,
    pub is_revoked: Option<bool>,
}

#[derive(Serialize, Debug, sqlx::FromRow, Clone)]
pub struct UserIdentityRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub provider_subject: String,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub picture_url: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}
