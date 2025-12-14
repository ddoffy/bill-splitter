use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

// Initialize panic hook for better error messages
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// ============================================================================
// Data Models (matching server-side models)
// ============================================================================

fn default_quantity() -> u32 {
    1
}

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

struct PersonSummary {
    name: String,
    amount_spent: f64,
    tip: f64,
    sponsor_amount: f64,
    is_sponsor: bool,
    is_receiver: bool,
    will_receive_from_others: f64,
    owes_to_others: f64,
    delegated_self: f64,
}

// ============================================================================
// WASM-Exposed Functions
// ============================================================================

/// Calculate bill split - main entry point for JavaScript
/// 
/// Takes a JSON string of CalculateRequest and returns JSON string of CalculateResponse
#[wasm_bindgen]
pub fn calculate_split(request_json: &str) -> Result<String, JsValue> {
    let request: CalculateRequest = serde_json::from_str(request_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse request: {}", e)))?;
    
    let response = calculate_split_internal(request);
    
    serde_json::to_string(&response)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize response: {}", e)))
}

/// Get WASM module version
#[wasm_bindgen]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Check if WASM module is loaded and working
#[wasm_bindgen]
pub fn health_check() -> bool {
    true
}

// ============================================================================
// Core Calculation Logic (identical to server-side)
// ============================================================================

fn calculate_split_internal(request: CalculateRequest) -> CalculateResponse {
    let people = request.people;
    let include_sponsor = request.include_sponsor;
    let _restrict_sponsor = request.restrict_sponsor_to_spent.unwrap_or(true);
    let fund_amount = request.fund_amount;
    let tip_percentage = request.tip_percentage;

    // Group people by name to handle multiple entries for the same person
    let mut grouped_people: HashMap<String, PersonSummary> = HashMap::new();

    for person in &people {
        let entry = grouped_people.entry(person.name.clone()).or_insert(PersonSummary {
            name: person.name.clone(),
            amount_spent: 0.0,
            tip: 0.0,
            sponsor_amount: 0.0,
            is_sponsor: false,
            is_receiver: false,
            will_receive_from_others: 0.0,
            owes_to_others: 0.0,
            delegated_self: 0.0,
        });

        // Handle sponsor status
        entry.sponsor_amount += person.sponsor_amount;
        if person.is_sponsor {
            entry.is_sponsor = true;
        }
        if person.is_receiver {
            entry.is_receiver = true;
        }

        // Track expenses with "paid_by" set
        if let Some(ref payer_name) = person.paid_by {
            let total_expense = (person.amount_spent * person.quantity as f64) + person.tip;
            
            if payer_name == &person.name {
                // Self-payment: This is a private expense, exclude from shared pool
                entry.delegated_self += total_expense;
                // Also add to amount_spent since they're paying for it
                entry.amount_spent += person.amount_spent * person.quantity as f64;
                entry.tip += person.tip;
            } else {
                // Someone else will reimburse this person for this expense
                // The current person (who actually paid) will receive reimbursement
                entry.will_receive_from_others += total_expense;
                // DON'T add to amount_spent - it's offset by the reimbursement
                
                // The designated payer owes this amount
                let payer_entry = grouped_people.entry(payer_name.clone()).or_insert(PersonSummary {
                    name: payer_name.clone(),
                    amount_spent: 0.0,
                    tip: 0.0,
                    sponsor_amount: 0.0,
                    is_sponsor: false,
                    is_receiver: false,
                    will_receive_from_others: 0.0,
                    owes_to_others: 0.0,
                    delegated_self: 0.0,
                });
                payer_entry.owes_to_others += total_expense;
            }
        } else {
            // No paid_by: Normal shared expense
            entry.amount_spent += person.amount_spent * person.quantity as f64;
            entry.tip += person.tip;
        }
    }

    // Convert map to vector for processing
    let unique_people: Vec<PersonSummary> = grouped_people.into_values().collect();

    let tip_multiplier = 1.0 + (tip_percentage / 100.0);

    // Calculate totals with tip included (as if it's a tax)
    let total_spent_base: f64 = unique_people.iter().map(|p| p.amount_spent).sum();
    let total_explicit_tip: f64 = unique_people.iter().map(|p| p.tip).sum();
    
    // Total spent with tip = (Base * Global Tax) + Explicit Tips
    let total_spent_with_tip = (total_spent_base * tip_multiplier) + total_explicit_tip;
    
    // Calculate all delegated expenses (only private expenses paid_by self)
    let all_delegated_expenses: f64 = unique_people.iter().map(|p| p.delegated_self).sum();
    
    // Sponsorship is a fixed amount, not affected by tip/tax
    let total_sponsored: f64 = unique_people.iter().map(|p| p.sponsor_amount).sum();
    
    // Handle sponsorship logic based on restriction setting
    let (effective_total_sponsored, sponsorship_ratio) = if total_sponsored > total_spent_with_tip {
        let ratio = if total_sponsored > 0.0 { total_spent_with_tip / total_sponsored } else { 0.0 };
        (total_spent_with_tip, ratio)
    } else {
        (total_sponsored, 1.0)
    };
    
    // The amount that needs to be shared among participants
    let amount_to_share = (total_spent_with_tip - effective_total_sponsored - fund_amount - all_delegated_expenses).max(0.0);

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
            let global_tip_part = if tip_percentage > 0.0 {
                person.amount_spent * (tip_percentage / 100.0)
            } else {
                0.0
            };
            
            let tip_paid = person.tip + global_tip_part;

            let sponsor_cost = if person.is_sponsor {
                person.sponsor_amount * sponsorship_ratio
            } else {
                0.0
            };
            
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

            let total_cost = sponsor_cost + share_cost + person.delegated_self + person.owes_to_others;
            let balance = (person.amount_spent + tip_paid + person.will_receive_from_others) - total_cost;

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
                tip_paid,
                sponsor_cost,
                share_cost,
                balance,
                settlement_type,
                is_receiver: person.is_receiver,
            }
        })
        .collect();
    
    // Sort settlements: Payers (negative balance) first, then Receivers (positive balance)
    settlements.sort_by(|a, b| {
        a.balance.partial_cmp(&b.balance)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.name.cmp(&b.name))
    });

    CalculateResponse {
        total_spent: total_spent_base + total_explicit_tip,
        total_sponsored: effective_total_sponsored,
        fund_amount,
        total_tip: total_spent_with_tip - (total_spent_base + total_explicit_tip),
        amount_to_share,
        num_participants,
        per_person_share,
        settlements,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_split() {
        let request = CalculateRequest {
            people: vec![
                Person {
                    id: 1,
                    name: "Alice".to_string(),
                    description: "Dinner".to_string(),
                    amount_spent: 100.0,
                    quantity: 1,
                    tip: 0.0,
                    is_sponsor: false,
                    sponsor_amount: 0.0,
                    is_receiver: false,
                    paid_by: None,
                },
                Person {
                    id: 2,
                    name: "Bob".to_string(),
                    description: "".to_string(),
                    amount_spent: 0.0,
                    quantity: 1,
                    tip: 0.0,
                    is_sponsor: false,
                    sponsor_amount: 0.0,
                    is_receiver: false,
                    paid_by: None,
                },
            ],
            include_sponsor: true,
            restrict_sponsor_to_spent: Some(true),
            fund_amount: 0.0,
            tip_percentage: 0.0,
        };

        let response = calculate_split_internal(request);
        
        assert_eq!(response.total_spent, 100.0);
        assert_eq!(response.per_person_share, 50.0);
        assert_eq!(response.settlements.len(), 2);
    }

    #[test]
    fn test_wasm_interface() {
        let json = r#"{
            "people": [
                {"id": 1, "name": "Alice", "description": "", "amount_spent": 100, "quantity": 1, "tip": 0, "is_sponsor": false, "sponsor_amount": 0, "is_receiver": false}
            ],
            "include_sponsor": true,
            "fund_amount": 0,
            "tip_percentage": 0
        }"#;

        let result = calculate_split(json);
        assert!(result.is_ok());
        
        let response: CalculateResponse = serde_json::from_str(&result.unwrap()).unwrap();
        assert_eq!(response.total_spent, 100.0);
    }
}
