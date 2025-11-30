use core::panic;

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use serde_json::json;
use base64::{Engine as _, engine::general_purpose};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReceiptItem {
    pub name: String,
    pub amount: f64,
    #[serde(default = "default_quantity")]
    pub quantity: u32,
}

fn default_quantity() -> u32 {
    1
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReceiptData {
    pub description: String,
    pub amount: f64,
    #[serde(default)]
    pub tip: Option<f64>,
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default)]
    pub items: Vec<ReceiptItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiExpense {
    pub name: String,
    pub description: String,
    pub amount_spent: f64,
    #[serde(default = "default_quantity")]
    pub quantity: u32,
    #[serde(default)]
    pub tip: f64,
    #[serde(default)]
    pub is_sponsor: bool,
    #[serde(default)]
    pub sponsor_amount: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiSplitResponse {
    pub expenses: Vec<AiExpense>,
    #[serde(default)]
    pub fund_amount: Option<f64>,
}

#[async_trait]
pub trait AiProvider: Send + Sync {
    async fn process_text(&self, text: &str) -> Result<ReceiptData, String>;
    async fn process_split_text(&self, text: &str) -> Result<AiSplitResponse, String>;
    async fn process_image(&self, image_data: &[u8], mime_type: &str) -> Result<ReceiptData, String>;
}

pub struct OpenAiProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl OpenAiProvider {
    pub fn new(api_key: String) -> Self {
        let model = std::env::var("OPENAI_API_MODEL").unwrap_or_default();

        if model.is_empty() {
            panic!("OPENAI_API_MODEL environment variable is not set");
        }

        Self {
            client: Client::new(),
            api_key,
            model,
        }
    }

    async fn make_request<T: DeserializeOwned>(&self, messages: Vec<serde_json::Value>) -> Result<T, String> {
        // read temperature from env or default to 0.5
        let temperature: f64 = std::env::var("OPENAI_API_TEMPERATURE")
            .unwrap_or_else(|_| "0.5".into())
            .parse()
            .unwrap_or(0.5);

        let payload = json!({
            "model": self.model,
            "messages": messages,
            "response_format": { "type": "json_object" },
            "temperature": temperature
        });

        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("OpenAI API error: {}", error_text));
        }

        let response_body: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
        
        let content = response_body["choices"][0]["message"]["content"]
            .as_str()
            .ok_or("No content in response")?;

        let data: T = serde_json::from_str(content).map_err(|e| format!("Failed to parse JSON: {} - Content: {}", e, content))?;
        
        Ok(data)
    }
}

#[async_trait]
impl AiProvider for OpenAiProvider {
    async fn process_text(&self, text: &str) -> Result<ReceiptData, String> {
        let system_prompt = "You are a helpful assistant that extracts receipt data. The text may be in English or Vietnamese. Output JSON matching the schema: description, amount (total), tip (optional), date (YYYY-MM-DD), and items (list of name/amount/quantity where quantity defaults to 1).";
        
        let messages = vec![
            json!({ "role": "system", "content": system_prompt }),
            json!({ "role": "user", "content": text }),
        ];

        self.make_request(messages).await
    }

    async fn process_split_text(&self, text: &str) -> Result<AiSplitResponse, String> {
        let system_prompt = "You are a helpful assistant that parses expense descriptions. \
            The text may be in English, Vietnamese, or a mix of both. \
            Extract a list of expenses from the text. \
            For each expense, identify the person's name, a description of what they paid for, the amount spent, quantity (defaults to 1), and any tip amount if specified. \
            Also detect if the person is 'sponsoring' the amount (paying for everyone without expecting repayment). \
            Vietnamese terms like 'tài trợ', 'bao', 'mời', 'sponsor' indicate sponsoring. (e.g. Anh sponsor 500k, it means Anh is paying 500k for everyone, spent will be 0, sponsor will be 500k) \
            Sometimes the sponsor amount may be different from the amount spent, they flexiblely use 'đ', '$', 'k', 'ngàn', 'vnd' for money unit, so capture both values if possible. \
            Also detect if there is a general 'fund', 'deposit', or 'quỹ' mentioned (amount available to cover expenses). \
            Handle Vietnamese terms like 'trả' (paid), 'mua' (bought), 'tiền' (money), 'k' (thousand, e.g. 50k = 50000), 'tài trợ' (sponsor), 'bao' (treat/sponsor), 'mời' (treat), 'quỹ' (fund), 'đóng quỹ' (deposit). \
            Output JSON matching the schema: { \"expenses\": [ { \"name\": string, \"description\": string, \"amount_spent\": number, \"quantity\": number (defaults to 1), \"tip\": number, \"is_sponsor\": boolean, \"sponsor_amount\": number } ], \"fund_amount\": number (optional) }. \
            If tip is not specified, set it to 0. If description is not clear, use a generic one like 'Expense'. \
            If is_sponsor is true, set sponsor_amount to the amount_spent unless specified otherwise.";
        
        let messages = vec![
            json!({ "role": "system", "content": system_prompt }),
            json!({ "role": "user", "content": text }),
        ];

        self.make_request(messages).await
    }

    async fn process_image(&self, image_data: &[u8], mime_type: &str) -> Result<ReceiptData, String> {
        let base64_image = general_purpose::STANDARD.encode(image_data);
        let data_url = format!("data:{};base64,{}", mime_type, base64_image);

                let system_prompt = "You are a helpful assistant that extracts receipt data from images. The receipt may be in English or Vietnamese. Output JSON matching the schema: description, amount (total), tip (optional), date (YYYY-MM-DD), and items (list of name/amount/quantity where quantity defaults to 1).";

        let messages = vec![
            json!({ "role": "system", "content": system_prompt }),
            json!({
                "role": "user",
                "content": [
                    { "type": "text", "text": "Extract data from this receipt." },
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": data_url
                        }
                    }
                ]
            }),
        ];

        self.make_request(messages).await
    }
}
