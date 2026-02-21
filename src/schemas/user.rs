use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Debug)]
pub struct UserProfile {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub base_currency: String,
    pub joined_at: DateTime<Utc>,
}
