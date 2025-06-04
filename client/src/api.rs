use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ApiError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("Authentication failed")]
    Unauthorized,
    #[error("Server error: {0}")]
    Server(String),
}

#[derive(Serialize)]
pub struct AuthRequest {
    pub password: String,
}

#[derive(Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub expires_in: u64,
}

#[derive(Deserialize)]
pub struct ProvidersResponse {
    pub providers: Vec<ProviderInfo>,
}

#[derive(Deserialize, Clone)]
pub struct ProviderInfo {
    pub name: String,
    pub models: Vec<String>,
}

#[derive(Serialize)]
pub struct ChatRequest {
    pub provider: String,
    pub messages: Vec<ChatMessage>,
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<usize>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct ChatChunk {
    pub text: String,
    pub done: bool,
}

pub struct ApiClient {
    base_url: String,
}

impl ApiClient {
    pub fn new() -> Self {
        // In production, this would come from config
        Self {
            base_url: "/api".to_string(),
        }
    }
    
    pub async fn authenticate(&self, password: String) -> Result<AuthResponse, ApiError> {
        let response = Request::post(&format!("{}/auth", self.base_url))
            .json(&AuthRequest { password })
            .map_err(|e| ApiError::Network(e.to_string()))?
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        
        if response.status() == 401 {
            return Err(ApiError::Unauthorized);
        }
        
        if !response.ok() {
            return Err(ApiError::Server(format!("Status: {}", response.status())));
        }
        
        response
            .json::<AuthResponse>()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))
    }
    
    pub async fn list_providers(&self, token: &str) -> Result<ProvidersResponse, ApiError> {
        let response = Request::get(&format!("{}/providers", self.base_url))
            .header("Authorization", &format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        
        if response.status() == 401 {
            return Err(ApiError::Unauthorized);
        }
        
        if !response.ok() {
            return Err(ApiError::Server(format!("Status: {}", response.status())));
        }
        
        response
            .json::<ProvidersResponse>()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))
    }
    
    pub fn chat_url(&self) -> String {
        format!("{}/chat", self.base_url)
    }
}