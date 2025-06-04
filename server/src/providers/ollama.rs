use anyhow::Result;
use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::{ChatChunk, ChatRequest, ChatStream, InferenceProvider};
use crate::config::OllamaConfig;

pub struct OllamaProvider {
    config: OllamaConfig,
    client: Client,
}

#[derive(Serialize)]
struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<usize>,
}

#[derive(Deserialize)]
struct OllamaGenerateResponse {
    response: String,
    done: bool,
}

#[derive(Deserialize)]
struct OllamaModelResponse {
    models: Vec<OllamaModel>,
}

#[derive(Deserialize)]
struct OllamaModel {
    name: String,
}

impl OllamaProvider {
    pub fn new(config: OllamaConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .unwrap_or_default();
        
        Self { config, client }
    }
    
    fn format_prompt(&self, request: &ChatRequest) -> String {
        // For now, simple formatting. Can be enhanced based on model requirements
        let mut prompt = String::new();
        
        for message in &request.messages {
            match message.role.as_str() {
                "user" => prompt.push_str(&format!("Question: {}\n\n", message.content)),
                "assistant" => prompt.push_str(&format!("Charles Fort: {}\n\n", message.content)),
                _ => {}
            }
        }
        
        // Add prefix for the response
        prompt.push_str("Charles Fort: ");
        prompt
    }
}

#[async_trait]
impl InferenceProvider for OllamaProvider {
    fn name(&self) -> &str {
        "ollama"
    }
    
    async fn available(&self) -> bool {
        // Check if Ollama is running by trying to list models
        self.list_models().await.is_ok()
    }
    
    async fn list_models(&self) -> Result<Vec<String>> {
        let url = format!("{}/api/tags", self.config.base_url);
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            anyhow::bail!("Failed to list Ollama models: {}", response.status());
        }
        
        let models: OllamaModelResponse = response.json().await?;
        Ok(models.models.into_iter().map(|m| m.name).collect())
    }
    
    async fn chat(&self, request: ChatRequest) -> Result<ChatStream> {
        let model = request.model
            .as_ref()
            .unwrap_or(&self.config.default_model)
            .to_string();
        
        let prompt = self.format_prompt(&request);
        
        let ollama_request = OllamaGenerateRequest {
            model,
            prompt,
            stream: true,
            options: Some(OllamaOptions {
                temperature: request.temperature,
                num_predict: request.max_tokens,
            }),
        };
        
        let url = format!("{}/api/generate", self.config.base_url);
        let response = self.client
            .post(&url)
            .json(&ollama_request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            anyhow::bail!("Ollama request failed: {}", response.status());
        }
        
        let stream = response
            .bytes_stream()
            .map(move |chunk| -> Result<ChatChunk> {
                let chunk = chunk?;
                let line = String::from_utf8_lossy(&chunk);
                
                // Ollama sends newline-delimited JSON
                for json_line in line.lines() {
                    if json_line.trim().is_empty() {
                        continue;
                    }
                    
                    if let Ok(resp) = serde_json::from_str::<OllamaGenerateResponse>(json_line) {
                        return Ok(ChatChunk {
                            text: resp.response,
                            done: resp.done,
                        });
                    }
                }
                
                Ok(ChatChunk {
                    text: String::new(),
                    done: false,
                })
            })
            .filter(|result| {
                futures::future::ready(match result {
                    Ok(chunk) => !chunk.text.is_empty() || chunk.done,
                    Err(_) => true,
                })
            });
        
        Ok(Box::pin(stream))
    }
}