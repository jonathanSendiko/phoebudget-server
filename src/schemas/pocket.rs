use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::common::round_currency;

#[derive(Deserialize, Debug)]
pub struct CreatePocket {
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct UpdatePocket {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct Pocket {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub icon: String,
    pub is_default: bool,
    pub created_at: Option<DateTime<Utc>>,
}

/// Detailed pocket info including balance (for get_pocket endpoint)
#[derive(Serialize, Debug)]
pub struct PocketDetail {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub icon: String,
    pub is_default: bool,
    pub created_at: Option<DateTime<Utc>>,
    #[serde(serialize_with = "round_currency")]
    pub balance: Decimal,
}

/// Lightweight pocket info for embedding in transactions
#[derive(Serialize, Debug, Clone)]
pub struct PocketSummary {
    pub id: Uuid,
    pub name: String,
    pub icon: String,
}

#[derive(Serialize)]
pub struct PocketId {
    pub id: Uuid,
}
