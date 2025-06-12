use anyhow::Result;
use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json;
use std::time::Duration;
use tracing;

use super::{ChatChunk, ChatRequest, ChatStream, InferenceProvider};
use crate::config::OllamaConfig;

pub struct OllamaProvider {
    config: OllamaConfig,
    client: Client,
}

#[derive(Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaChatMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Serialize)]
struct OllamaChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<usize>,
}

#[derive(Deserialize)]
struct OllamaChatResponse {
    message: Option<OllamaChatResponseMessage>,
    done: bool,
}

#[derive(Deserialize)]
struct OllamaChatResponseMessage {
    role: String,
    content: String,
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
        
        tracing::info!("Ollama chat request for model: {}", model);
        
        // Build messages array with system prompt if provided
        let mut messages = Vec::new();
        
        // Add system message if provided
        if let Some(system) = &request.system_prompt {
            messages.push(OllamaChatMessage {
                role: "system".to_string(),
                content: system.clone(),
            });
        }
        
        // Add user messages
        for msg in &request.messages {
            messages.push(OllamaChatMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
            });
        }
        
        let ollama_request = OllamaChatRequest {
            model,
            messages,
            stream: true,
            options: Some(OllamaOptions {
                temperature: request.temperature,
                num_predict: request.max_tokens,
            }),
        };
        
        let url = format!("{}/api/chat", self.config.base_url);
        tracing::info!("Sending request to: {}", url);
        
        let response = self.client
            .post(&url)
            .json(&ollama_request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            anyhow::bail!("Ollama request failed: {}", response.status());
        }
        
        tracing::info!("Ollama response status: {}", response.status());
        
        use std::sync::{Arc, Mutex};
        
        // Track if we've seen certain stop patterns
        let response_buffer = Arc::new(Mutex::new(String::new()));
        let stop_streaming = Arc::new(Mutex::new(false));
        
        let stream = response
            .bytes_stream()
            .map(move |chunk| -> Result<ChatChunk> {
                let chunk = chunk?;
                let line = String::from_utf8_lossy(&chunk);
                
                tracing::debug!("Received chunk: {}", line);
                
                // Ollama sends newline-delimited JSON
                for json_line in line.lines() {
                    if json_line.trim().is_empty() {
                        continue;
                    }
                    
                    match serde_json::from_str::<OllamaChatResponse>(json_line) {
                        Ok(resp) => {
                            // Extract content from message
                            let content = resp.message
                                .map(|m| m.content)
                                .unwrap_or_default();
                            
                            tracing::debug!("Parsed response: text='{}', done={}", content, resp.done);
                            
                            // Check if we should stop
                            let mut should_stop = stop_streaming.lock().unwrap();
                            if *should_stop {
                                return Ok(ChatChunk {
                                    text: String::new(),
                                    done: true,
                                });
                            }
                            
                            // Accumulate response
                            let mut buffer = response_buffer.lock().unwrap();
                            buffer.push_str(&content);
                            
                            // Check for generic stop patterns
                            if buffer.contains("\n---\n") || 
                               buffer.contains("\nUser:") || 
                               buffer.contains("\n\nUser:") ||
                               buffer.contains("\nHuman:") || 
                               buffer.contains("\n\nHuman:") {
                                *should_stop = true;
                                return Ok(ChatChunk {
                                    text: content,
                                    done: true,
                                });
                            }
                            
                            return Ok(ChatChunk {
                                text: content,
                                done: resp.done,
                            });
                        }
                        Err(e) => {
                            tracing::error!("Failed to parse Ollama response: {}, line: {}", e, json_line);
                        }
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