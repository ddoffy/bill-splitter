use core::panic;

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use base64::{Engine as _, engine::general_purpose};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReceiptItem {
    pub name: String,
    pub amount: f64,
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

#[async_trait]
pub trait AiProvider: Send + Sync {
    async fn process_text(&self, text: &str) -> Result<ReceiptData, String>;
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

    async fn make_request(&self, messages: Vec<serde_json::Value>) -> Result<ReceiptData, String> {
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

        let data: ReceiptData = serde_json::from_str(content).map_err(|e| format!("Failed to parse JSON: {} - Content: {}", e, content))?;
        
        Ok(data)
    }
}

#[async_trait]
impl AiProvider for OpenAiProvider {
    async fn process_text(&self, text: &str) -> Result<ReceiptData, String> {
        let system_prompt = "You are a helpful assistant that extracts receipt data. Output JSON matching the schema: description, amount (total), tip (optional), date (YYYY-MM-DD), and items (list of name/amount).";
        
        let messages = vec![
            json!({ "role": "system", "content": system_prompt }),
            json!({ "role": "user", "content": text }),
        ];

        self.make_request(messages).await
    }

    async fn process_image(&self, image_data: &[u8], mime_type: &str) -> Result<ReceiptData, String> {
        let base64_image = general_purpose::STANDARD.encode(image_data);
        let data_url = format!("data:{};base64,{}", mime_type, base64_image);

        let system_prompt = "You are a helpful assistant that extracts receipt data from images. Output JSON matching the schema: description, amount (total), tip (optional), date (YYYY-MM-DD), and items (list of name/amount).";

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
