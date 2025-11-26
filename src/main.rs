use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use askama::Template;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions, FromRow};

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Person {
    id: u64,
    name: String,
    description: String,
    amount_spent: f64,
    is_sponsor: bool,
    sponsor_amount: f64,
    #[serde(default)]
    is_receiver: bool,
}

#[derive(Debug, FromRow)]
struct DbSession {
    id: String,
    edit_secret: String,
    people: String,
    created_at: DateTime<Utc>,
    last_accessed_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateSessionRequest {
    people: Vec<Person>,
}

#[derive(Debug, Serialize)]
struct CreateSessionResponse {
    id: String,
    edit_secret: String,
}

#[derive(Debug, Serialize)]
struct GetSessionResponse {
    people: Vec<Person>,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateSessionRequest {
    people: Vec<Person>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CalculateRequest {
    people: Vec<Person>,
    include_sponsor: bool,
    restrict_sponsor_to_spent: Option<bool>,
}

#[derive(Debug, Serialize)]
struct Settlement {
    name: String,
    amount_spent: f64,
    sponsor_cost: f64,
    share_cost: f64,
    balance: f64,
    settlement_type: String,
    is_receiver: bool,
}

#[derive(Debug, Serialize)]
struct CalculateResponse {
    total_spent: f64,
    total_sponsored: f64,
    amount_to_share: f64,
    num_participants: usize,
    per_person_share: f64,
    settlements: Vec<Settlement>,
}

type AppState = SqlitePool;

const SESSION_EXPIRY_DAYS: i64 = 7;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "split_bills=debug,tower_http=debug,axum=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_url = "sqlite:sessions.db?mode=rwc";
    let pool = SqlitePoolOptions::new()
        .connect(db_url)
        .await
        .expect("Failed to connect to database");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            edit_secret TEXT NOT NULL,
            people TEXT NOT NULL,
            created_at DATETIME NOT NULL,
            last_accessed_at DATETIME NOT NULL
        )
        "#
    )
    .execute(&pool)
    .await
    .expect("Failed to create table");

    let cleanup_pool = pool.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await; // Check every hour
            cleanup_expired_sessions(&cleanup_pool).await;
        }
    });

    let app = Router::new()
        .route("/", get(index))
        .route("/api/calculate", post(calculate_split))
        .route("/api/sessions", post(create_session))
        .route("/api/sessions/:id", get(get_session).put(update_session))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7777")
        .await
        .unwrap();
    
    tracing::info!("Server running on http://0.0.0.0:7777");
    
    axum::serve(listener, app).await.unwrap();
}

async fn cleanup_expired_sessions(pool: &SqlitePool) {
    let threshold = Utc::now() - chrono::Duration::days(SESSION_EXPIRY_DAYS);
    let result = sqlx::query("DELETE FROM sessions WHERE last_accessed_at < ?")
        .bind(threshold)
        .execute(pool)
        .await;
    
    match result {
        Ok(r) => {
            if r.rows_affected() > 0 {
                tracing::info!("Cleaned up {} expired sessions", r.rows_affected());
            }
        }
        Err(e) => tracing::error!("Failed to cleanup sessions: {}", e),
    }
}

async fn index() -> impl IntoResponse {
    Html(IndexTemplate.render().unwrap())
}

async fn create_session(
    State(pool): State<AppState>,
    Json(request): Json<CreateSessionRequest>,
) -> Json<CreateSessionResponse> {
    let id = Uuid::new_v4().to_string();
    let edit_secret = Uuid::new_v4().to_string();
    let now = Utc::now();
    let people_json = serde_json::to_string(&request.people).unwrap_or_default();
    
    sqlx::query(
        "INSERT INTO sessions (id, edit_secret, people, created_at, last_accessed_at) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&edit_secret)
    .bind(&people_json)
    .bind(now)
    .bind(now)
    .execute(&pool)
    .await
    .unwrap();
    
    Json(CreateSessionResponse {
        id,
        edit_secret,
    })
}

async fn get_session(
    State(pool): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<GetSessionResponse>, axum::http::StatusCode> {
    let now = Utc::now();
    
    let update_result = sqlx::query("UPDATE sessions SET last_accessed_at = ? WHERE id = ?")
        .bind(now)
        .bind(&id)
        .execute(&pool)
        .await;

    if let Err(_) = update_result {
        return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    let row: Option<DbSession> = sqlx::query_as("SELECT * FROM sessions WHERE id = ?")
        .bind(&id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(session) = row {
        let people: Vec<Person> = serde_json::from_str(&session.people)
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
            
        Ok(Json(GetSessionResponse {
            people,
        }))
    } else {
        Err(axum::http::StatusCode::NOT_FOUND)
    }
}

async fn update_session(
    State(pool): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    headers: axum::http::HeaderMap,
    Json(request): Json<UpdateSessionRequest>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let secret_header = headers.get("X-Edit-Secret")
        .and_then(|h| h.to_str().ok());
        
    if let Some(secret) = secret_header {
        let row: Option<(String,)> = sqlx::query_as("SELECT edit_secret FROM sessions WHERE id = ?")
            .bind(&id)
            .fetch_optional(&pool)
            .await
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
            
        if let Some((stored_secret,)) = row {
            if stored_secret == secret {
                let people_json = serde_json::to_string(&request.people)
                    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
                let now = Utc::now();
                
                sqlx::query("UPDATE sessions SET people = ?, last_accessed_at = ? WHERE id = ?")
                    .bind(people_json)
                    .bind(now)
                    .bind(&id)
                    .execute(&pool)
                    .await
                    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
                    
                return Ok(Json(serde_json::json!({"success": true})));
            } else {
                return Err(axum::http::StatusCode::FORBIDDEN);
            }
        } else {
            return Err(axum::http::StatusCode::NOT_FOUND);
        }
    }
    Err(axum::http::StatusCode::FORBIDDEN)
}

async fn calculate_split(Json(request): Json<CalculateRequest>) -> Json<CalculateResponse> {
    let people = request.people;
    let include_sponsor = request.include_sponsor;
    let restrict_sponsor = request.restrict_sponsor_to_spent.unwrap_or(true);

    // Group people by name to handle multiple entries for the same person
    struct PersonSummary {
        name: String,
        amount_spent: f64,
        sponsor_amount: f64,
        is_sponsor: bool,
        is_receiver: bool,
    }

    let mut grouped_people: HashMap<String, PersonSummary> = HashMap::new();

    for person in &people {
        let entry = grouped_people.entry(person.name.clone()).or_insert(PersonSummary {
            name: person.name.clone(),
            amount_spent: 0.0,
            sponsor_amount: 0.0,
            is_sponsor: false,
            is_receiver: false,
        });

        entry.amount_spent += person.amount_spent;
        entry.sponsor_amount += person.sponsor_amount;
        if person.is_sponsor {
            entry.is_sponsor = true;
        }
        if person.is_receiver {
            entry.is_receiver = true;
        }
    }

    // Convert map to vector for processing
    let unique_people: Vec<PersonSummary> = grouped_people.into_values().collect();

    let total_spent: f64 = unique_people.iter().map(|p| p.amount_spent).sum();
    let total_sponsored_raw: f64 = unique_people.iter().map(|p| p.sponsor_amount).sum();
    
    // Handle sponsorship logic based on restriction setting
    // Always restrict sponsorship to total spent to ensure no one profits (negative share)
    let (effective_total_sponsored, sponsorship_ratio) = if total_sponsored_raw > total_spent {
        // Restrict enabled: Cap sponsorship at total spent
        // Scale down sponsor contributions
        let ratio = if total_sponsored_raw > 0.0 { total_spent / total_sponsored_raw } else { 0.0 };
        (total_spent, ratio)
    } else {
        // Sponsorship <= spent: Use actual sponsorship amount
        (total_sponsored_raw, 1.0)
    };
    
    // The amount that needs to be shared among participants
    let amount_to_share = total_spent - effective_total_sponsored;

    let participants: Vec<&PersonSummary> = if include_sponsor {
        unique_people.iter().collect()
    } else {
        unique_people.iter().filter(|p| !p.is_sponsor).collect()
    };

    let num_participants = participants.len();
    let per_person_share = if num_participants > 0 {
        amount_to_share / num_participants as f64
    } else {
        0.0
    };

    let mut settlements: Vec<Settlement> = unique_people
        .iter()
        .map(|person| {
            // Calculate how much this person should pay (cost)
            let sponsor_cost = if person.is_sponsor {
                person.sponsor_amount * sponsorship_ratio
            } else {
                0.0
            };
            
            // If they are a participant in the split, add the shared amount
            let is_participant = if include_sponsor {
                true
            } else {
                !person.is_sponsor
            };

            let share_cost = if is_participant {
                per_person_share
            } else {
                0.0
            };

            let total_cost = sponsor_cost + share_cost;

            // Balance = What they paid (amount_spent) - What they should pay (total_cost)
            let balance = person.amount_spent - total_cost;

            let settlement_type = if balance > 0.01 {
                "receive".to_string()
            } else if balance < -0.01 {
                "pay".to_string()
            } else {
                "settled".to_string()
            };

            Settlement {
                name: person.name.clone(),
                amount_spent: person.amount_spent,
                sponsor_cost,
                share_cost,
                balance,
                settlement_type,
                is_receiver: person.is_receiver,
            }
        })
        .collect();
    
    // Sort settlements by name for consistent display
    settlements.sort_by(|a, b| a.name.cmp(&b.name));

    Json(CalculateResponse {
        total_spent,
        total_sponsored: effective_total_sponsored,
        amount_to_share,
        num_participants,
        per_person_share,
        settlements,
    })
}
