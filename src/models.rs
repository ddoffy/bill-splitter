use askama::Template;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

// Template structs
#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub base_url: String,
}

// Helper functions
pub fn default_quantity() -> u32 {
    1
}

// Core data models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub amount_spent: f64,
    #[serde(default = "default_quantity")]
    pub quantity: u32,
    #[serde(default)]
    pub tip: f64,
    pub is_sponsor: bool,
    pub sponsor_amount: f64,
    #[serde(default)]
    pub is_receiver: bool,
    #[serde(default)]
    pub paid_by: Option<String>,
}

#[derive(Debug, FromRow)]
#[allow(dead_code)]
pub struct DbSession {
    pub id: String,
    pub edit_secret: String,
    pub people: String,
    pub created_at: DateTime<Utc>,
    pub last_accessed_at: DateTime<Utc>,
    #[sqlx(default)]
    pub fund_amount: f64,
    #[sqlx(default)]
    pub tip_percentage: f64,
}

// API request/response structs
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub people: Vec<Person>,
    #[serde(default)]
    pub fund_amount: f64,
    #[serde(default)]
    pub tip_percentage: f64,
}

#[derive(Debug, Serialize)]
pub struct CreateSessionResponse {
    pub id: String,
    pub edit_secret: String,
}

#[derive(Debug, Serialize)]
pub struct GetSessionResponse {
    pub people: Vec<Person>,
    pub fund_amount: f64,
    pub tip_percentage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSessionRequest {
    pub people: Vec<Person>,
    #[serde(default)]
    pub fund_amount: f64,
    #[serde(default)]
    pub tip_percentage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CalculateRequest {
    pub people: Vec<Person>,
    pub include_sponsor: bool,
    pub restrict_sponsor_to_spent: Option<bool>,
    #[serde(default)]
    pub fund_amount: f64,
    #[serde(default)]
    pub tip_percentage: f64,
}

#[derive(Debug, Serialize)]
pub struct CalculateResponse {
    pub total_spent: f64,
    pub total_sponsored: f64,
    pub fund_amount: f64,
    pub total_tip: f64,
    pub amount_to_share: f64,
    pub num_participants: usize,
    pub per_person_share: f64,
    pub settlements: Vec<Settlement>,
}

// Calculation structs
#[derive(Debug, Serialize)]
pub struct Settlement {
    pub name: String,
    pub amount_spent: f64,
    pub tip_paid: f64,
    pub sponsor_cost: f64,
    pub share_cost: f64,
    pub balance: f64,
    pub settlement_type: String,
    pub is_receiver: bool,
}

pub struct PersonSummary {
    pub name: String,
    pub amount_spent: f64,
    pub tip: f64,
    pub sponsor_amount: f64,
    pub is_sponsor: bool,
    pub is_receiver: bool,
    pub will_receive_from_others: f64,  // Amount they will receive as reimbursement
    pub owes_to_others: f64, // Amount they owe to reimburse others
    pub delegated_self: f64, // Amount they marked as paid_by themselves (private expense)
}

// Service request structs
#[derive(Deserialize)]
pub struct AiTextRequest {
    pub text: String,
}

#[derive(Deserialize)]
pub struct SendEmailRequest {
    pub to: Vec<String>,
    pub subject: String,
    pub html_content: String,
    pub cc: Option<Vec<String>>,
    pub bcc: Option<Vec<String>>,
}

// Application state
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub processed_requests: Arc<Mutex<HashMap<String, DateTime<Utc>>>>,
}
