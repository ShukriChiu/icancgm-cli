use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

pub const PROD_BASE_URL: &str = "https://ican.sinocare.com";
pub const API_BASE_PATH: &str = "/api/scrm-mcp";
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub timeout_secs: u64,
    pub personal_token: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            timeout_secs: DEFAULT_TIMEOUT_SECS,
            personal_token: std::env::var("SINO_PERSONAL_TOKEN").ok(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SinoClient {
    http: Client,
    config: AppConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiErrorResponse {
    pub error: String,
    pub message: String,
}

impl SinoClient {
    pub fn new(config: AppConfig) -> Result<Self> {
        let http = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()?;

        Ok(Self { http, config })
    }

    pub async fn health(&self) -> Result<Value> {
        self.get_json("/health", &[]).await
    }

    pub async fn user_info(&self, user_id: &str) -> Result<Value> {
        let path = format!("/api/user/{user_id}/info");
        self.get_json(&path, &[]).await
    }

    pub async fn cgm_day(&self, user_id: &str, date: &str) -> Result<Value> {
        let path = format!("/api/user/{user_id}/cgm-data");
        self.get_json(&path, &[("date", date)]).await
    }

    pub async fn cgm_range(&self, user_id: &str, start_date: &str, end_date: &str) -> Result<Value> {
        let path = format!("/api/user/{user_id}/cgm-data");
        self.get_json(&path, &[("start_date", start_date), ("end_date", end_date)])
            .await
    }

    pub async fn daily(&self, user_id: &str, date: &str) -> Result<Value> {
        let path = format!("/api/user/{user_id}/daily-data");
        self.get_json(&path, &[("date", date)]).await
    }

    pub async fn event(&self, user_id: &str, event_id: &str) -> Result<Value> {
        let path = format!("/api/user/{user_id}/event/{event_id}");
        self.get_json(&path, &[]).await
    }

    async fn get_json(&self, path: &str, query: &[(&str, &str)]) -> Result<Value> {
        let url = format!("{PROD_BASE_URL}{API_BASE_PATH}{path}");
        let mut request = self.http.get(url).query(query);

        if let Some(token) = &self.config.personal_token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;
        let status = response.status();

        if status.is_success() {
            return Ok(response.json::<Value>().await?);
        }

        let body = response.text().await?;
        if let Ok(api_error) = serde_json::from_str::<ApiErrorResponse>(&body) {
            return Err(anyhow!(
                "API request failed: status={} error={} message={}",
                status,
                api_error.error,
                api_error.message
            ));
        }

        Err(anyhow!("API request failed: status={} body={}", status, body))
    }
}
