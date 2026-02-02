use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::common::round_currency;
use super::pocket::PocketSummary;

// --- Financial Goals DTOs ---

#[derive(Deserialize, Debug)]
pub struct CreateGoal {
    pub name: String,
    pub description: Option<String>,
    pub target_amount: Decimal,
    pub current_amount: Option<Decimal>, // Manual amount, default 0
    pub pocket_id: Uuid,
    pub icon: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct UpdateGoal {
    pub name: Option<String>,
    pub description: Option<String>,
    pub target_amount: Option<Decimal>,
    pub current_amount: Option<Decimal>,
    pub icon: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct GoalSummary {
    pub id: Uuid,
    pub name: String,
    pub icon: String,
    #[serde(serialize_with = "round_currency")]
    pub target_amount: Decimal,
    #[serde(serialize_with = "round_currency")]
    pub current_amount: Decimal,
    #[serde(serialize_with = "round_currency")]
    pub percentage: Decimal,
}

#[derive(Serialize, Debug)]
pub struct GoalDetail {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub icon: String,
    #[serde(serialize_with = "round_currency")]
    pub target_amount: Decimal,
    #[serde(serialize_with = "round_currency")]
    pub current_amount: Decimal,
    #[serde(serialize_with = "round_currency")]
    pub percentage: Decimal,
    pub pocket: PocketSummary,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct GoalId {
    pub id: Uuid,
}

// --- Goal Entries DTOs ---

#[derive(Deserialize, Debug)]
pub struct CreateGoalEntry {
    pub amount: Decimal,
    pub description: Option<String>,
    pub date: Option<DateTime<Utc>>,
}

#[derive(Serialize, Debug)]
pub struct GoalEntry {
    pub id: Uuid,
    pub goal_id: Uuid,
    #[serde(serialize_with = "round_currency")]
    pub amount: Decimal,
    pub description: Option<String>,
    pub date: DateTime<Utc>,
}
