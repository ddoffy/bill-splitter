use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use askama::Template;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Session {
    people: Vec<Person>,
    edit_secret: String,
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

type AppState = Arc<Mutex<HashMap<String, Session>>>;

const DATA_FILE: &str = "sessions.json";
const SESSION_EXPIRY_DAYS: i64 = 30;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "split_bills=debug,tower_http=debug,axum=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let sessions = load_sessions();
    let state = Arc::new(Mutex::new(sessions));

    // Spawn a background task to clean up expired sessions periodically
    let cleanup_state = state.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await; // Check every hour
            cleanup_expired_sessions(&cleanup_state);
        }
    });

    let app = Router::new()
        .route("/", get(index))
        .route("/api/calculate", post(calculate_split))
        .route("/api/sessions", post(create_session))
        .route("/api/sessions/:id", get(get_session).put(update_session))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7777")
        .await
        .unwrap();
    
    tracing::info!("Server running on http://0.0.0.0:7777");
    
    axum::serve(listener, app).await.unwrap();
}

fn load_sessions() -> HashMap<String, Session> {
    if let Ok(file) = File::open(DATA_FILE) {
        let reader = BufReader::new(file);
        match serde_json::from_reader(reader) {
            Ok(sessions) => {
                tracing::info!("Loaded sessions from file");
                return sessions;
            }
            Err(e) => {
                tracing::error!("Failed to parse sessions file: {}", e);
            }
        }
    }
    HashMap::new()
}

fn save_sessions(sessions: &HashMap<String, Session>) {
    if let Ok(file) = File::create(DATA_FILE) {
        let writer = BufWriter::new(file);
        if let Err(e) = serde_json::to_writer(writer, sessions) {
            tracing::error!("Failed to save sessions to file: {}", e);
        }
    }
}

fn cleanup_expired_sessions(state: &AppState) {
    let mut sessions = state.lock().unwrap();
    let now = Utc::now();
    let initial_len = sessions.len();
    
    sessions.retain(|_, session| {
        let age = now.signed_duration_since(session.last_accessed_at);
        age.num_days() < SESSION_EXPIRY_DAYS
    });
    
    if sessions.len() < initial_len {
        tracing::info!("Cleaned up {} expired sessions", initial_len - sessions.len());
        save_sessions(&sessions);
    }
}

async fn index() -> impl IntoResponse {
    Html(IndexTemplate.render().unwrap())
}

async fn create_session(
    State(state): State<AppState>,
    Json(request): Json<CreateSessionRequest>,
) -> Json<CreateSessionResponse> {
    let mut sessions = state.lock().unwrap();
    let id = Uuid::new_v4().to_string();
    let edit_secret = Uuid::new_v4().to_string();
    let now = Utc::now();
    
    let session = Session {
        people: request.people,
        edit_secret: edit_secret.clone(),
        created_at: now,
        last_accessed_at: now,
    };
    
    sessions.insert(id.clone(), session);
    save_sessions(&sessions);
    
    Json(CreateSessionResponse {
        id,
        edit_secret,
    })
}

async fn get_session(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<GetSessionResponse>, axum::http::StatusCode> {
    let mut sessions = state.lock().unwrap();
    if let Some(session) = sessions.get_mut(&id) {
        // Update last accessed time
        session.last_accessed_at = Utc::now();
        // We don't save on every read to avoid excessive IO, 
        // but we could if strict accuracy is needed. 
        // For now, let's save periodically or on write.
        // Actually, let's save on read too to prevent premature expiry if only read.
        // To avoid blocking, we could clone and save in background, but for simplicity:
        let people = session.people.clone();
        
        // Drop lock before saving? No, save_sessions needs reference or clone.
        // Let's just save. It's a small file.
        save_sessions(&sessions);

        Ok(Json(GetSessionResponse {
            people,
        }))
    } else {
        Err(axum::http::StatusCode::NOT_FOUND)
    }
}

async fn update_session(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    headers: axum::http::HeaderMap,
    Json(request): Json<UpdateSessionRequest>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let mut sessions = state.lock().unwrap();
    
    if let Some(session) = sessions.get_mut(&id) {
        let secret_header = headers.get("X-Edit-Secret")
            .and_then(|h| h.to_str().ok());
            
        if let Some(secret) = secret_header {
            if secret == session.edit_secret {
                session.people = request.people;
                session.last_accessed_at = Utc::now();
                save_sessions(&sessions);
                return Ok(Json(serde_json::json!({"success": true})));
            }
        }
        Err(axum::http::StatusCode::FORBIDDEN)
    } else {
        Err(axum::http::StatusCode::NOT_FOUND)
    }
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
    let (effective_total_sponsored, sponsorship_ratio) = if restrict_sponsor && total_sponsored_raw > total_spent {
        // Restrict enabled: Cap sponsorship at total spent
        // Scale down sponsor contributions
        let ratio = if total_sponsored_raw > 0.0 { total_spent / total_sponsored_raw } else { 0.0 };
        (total_spent, ratio)
    } else {
        // Restrict disabled OR sponsorship <= spent: Use actual sponsorship amount
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
