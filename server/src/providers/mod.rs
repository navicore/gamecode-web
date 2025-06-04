use anyhow::Result;
use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;

pub mod ollama;

use crate::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,  // "user" or "assistant"
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChunk {
    pub text: String,
    pub done: bool,
}

pub type ChatStream = Pin<Box<dyn Stream<Item = Result<ChatChunk>> + Send>>;

#[async_trait]
pub trait InferenceProvider: Send + Sync {
    /// Get the name of this provider
    fn name(&self) -> &str;
    
    /// Check if the provider is available
    async fn available(&self) -> bool;
    
    /// List available models
    async fn list_models(&self) -> Result<Vec<String>>;
    
    /// Stream a chat response
    async fn chat(&self, request: ChatRequest) -> Result<ChatStream>;
}

pub struct ProviderManager {
    providers: HashMap<String, Box<dyn InferenceProvider>>,
}

impl ProviderManager {
    pub async fn new(config: &Config) -> Result<Self> {
        let mut providers = HashMap::new();
        
        // Initialize Ollama provider if configured
        if let Some(ollama_config) = &config.providers.ollama {
            if ollama_config.enabled {
                let ollama = ollama::OllamaProvider::new(ollama_config.clone());
                if ollama.available().await {
                    providers.insert("ollama".to_string(), Box::new(ollama) as Box<dyn InferenceProvider>);
                    tracing::info!("Ollama provider initialized");
                } else {
                    tracing::warn!("Ollama provider configured but not available");
                }
            }
        }
        
        // Future: Add other providers here
        
        if providers.is_empty() {
            anyhow::bail!("No inference providers available");
        }
        
        Ok(Self { providers })
    }
    
    pub fn get(&self, name: &str) -> Option<&Box<dyn InferenceProvider>> {
        self.providers.get(name)
    }
    
    pub fn list_available(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }
    
    pub async fn chat(&self, provider_name: &str, request: ChatRequest) -> Result<ChatStream> {
        let provider = self.get(provider_name)
            .ok_or_else(|| anyhow::anyhow!("Provider '{}' not found", provider_name))?;
        
        provider.chat(request).await
    }
}