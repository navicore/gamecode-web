use axum::{
    extract::State,
    response::{sse::Event, Sse},
    routing::{get, post},
    Json, Router,
};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, sync::Arc};
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::{
    auth::{verify_password, generate_token, AuthRequest, AuthResponse, AuthUser},
    error::AppError,
    providers::ChatRequest,
    AppState,
};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/health", get(health))
        .route("/auth", post(authenticate))
        .route("/providers", get(list_providers))
        .route("/chat", post(chat))
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    providers: Vec<ProviderStatus>,
}

#[derive(Serialize)]
struct ProviderStatus {
    name: String,
    available: bool,
}

async fn health(State(state): State<Arc<AppState>>) -> Result<Json<HealthResponse>, AppError> {
    let mut providers = Vec::new();
    
    for provider_name in state.providers.list_available() {
        if let Some(provider) = state.providers.get(&provider_name) {
            providers.push(ProviderStatus {
                name: provider_name,
                available: provider.available().await,
            });
        }
    }
    
    Ok(Json(HealthResponse {
        status: "ok".to_string(),
        providers,
    }))
}

async fn authenticate(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AuthRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // Debug log
    tracing::info!("Auth attempt with password length: {}", req.password.len());
    tracing::debug!("Expected hash: {}", &state.config.auth.password_hash);
    
    // Verify password
    verify_password(&req.password, &state.config.auth.password_hash)
        .map_err(|e| {
            tracing::warn!("Password verification failed: {:?}", e);
            AppError::Unauthorized
        })?;
    
    // Generate token
    let token = generate_token(&state.config.auth)?;
    
    Ok(Json(AuthResponse {
        token,
        expires_in: state.config.auth.session_duration_hours * 3600,
    }))
}

#[derive(Serialize)]
struct ProvidersResponse {
    providers: Vec<ProviderInfo>,
}

#[derive(Serialize)]
struct ProviderInfo {
    name: String,
    models: Vec<String>,
}

async fn list_providers(
    _auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<ProvidersResponse>, AppError> {
    let mut providers = Vec::new();
    
    for provider_name in state.providers.list_available() {
        if let Some(provider) = state.providers.get(&provider_name) {
            if let Ok(models) = provider.list_models().await {
                providers.push(ProviderInfo {
                    name: provider_name,
                    models,
                });
            }
        }
    }
    
    Ok(Json(ProvidersResponse { providers }))
}

#[derive(Deserialize)]
struct ChatRequestBody {
    provider: String,
    messages: Vec<crate::providers::ChatMessage>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    temperature: Option<f32>,
    #[serde(default)]
    max_tokens: Option<usize>,
}

async fn chat(
    _auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(req): Json<ChatRequestBody>,
) -> Result<Sse<UnboundedReceiverStream<Result<Event, Infallible>>>, AppError> {
    tracing::info!("Chat endpoint hit with provider: {}", req.provider);
    
    let chat_request = ChatRequest {
        messages: req.messages.clone(),
        model: req.model,
        temperature: req.temperature,
        max_tokens: req.max_tokens,
    };
    
    tracing::info!("Messages: {:?}", req.messages);
    
    let mut stream = state.providers
        .chat(&req.provider, chat_request)
        .await?;
    
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    
    // Spawn task to convert provider stream to SSE events
    tokio::spawn(async move {
        while let Some(result) = stream.next().await {
            match result {
                Ok(chunk) => {
                    let event = Event::default()
                        .data(serde_json::to_string(&chunk).unwrap_or_default());
                    
                    if tx.send(Ok(event)).is_err() {
                        break;
                    }
                    
                    if chunk.done {
                        break;
                    }
                }
                Err(e) => {
                    let error_event = Event::default()
                        .data(serde_json::to_string(&serde_json::json!({
                            "error": e.to_string()
                        })).unwrap_or_default());
                    
                    let _ = tx.send(Ok(error_event));
                    break;
                }
            }
        }
    });
    
    Ok(Sse::new(UnboundedReceiverStream::new(rx)))
}