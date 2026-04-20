use anyhow::anyhow;
use axum::{
    extract::{Query, State},
    http::{
        header::{COOKIE, SET_COOKIE},
        HeaderMap, HeaderValue, StatusCode,
    },
    middleware,
    response::{sse::Event, AppendHeaders, IntoResponse, Redirect, Response, Sse},
    routing::{get, post},
    Json, Router,
};
use cookie::Cookie;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::{
    convert::Infallible,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::{
    auth::{
        auth_middleware, clear_session_cookie, clear_tx_cookie,
        oidc::{pkce_challenge, random_b64_url},
        session::{open, seal},
        session_cookie, tx_cookie, AuthUser, SessionPayload, TxPayload, SESSION_COOKIE, TX_COOKIE,
    },
    error::AppError,
    providers::ChatRequest,
    AppState,
};
use std::fs;

pub fn routes(state: Arc<AppState>) -> Router {
    let public = Router::new()
        .route("/health", get(health))
        .route("/auth/login", get(auth_login))
        .route("/auth/callback", get(auth_callback))
        .route("/auth/logout", post(auth_logout))
        .with_state(state.clone());

    let protected = Router::new()
        .route("/me", get(me))
        .route("/providers", get(list_providers))
        .route("/prompts", get(list_prompts))
        .route("/chat", post(chat))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state);

    public.merge(protected)
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

async fn auth_login(State(state): State<Arc<AppState>>) -> Result<Response, AppError> {
    let code_verifier = random_b64_url(32);
    let code_challenge = pkce_challenge(&code_verifier);
    let state_tok = random_b64_url(16);
    let nonce = random_b64_url(16);

    let tx = TxPayload {
        code_verifier,
        state: state_tok.clone(),
        nonce: nonce.clone(),
    };
    let sealed = seal(&state.config.auth.session_key, TX_COOKIE.as_bytes(), &tx)?;
    let url = state
        .oidc
        .authorize_url(&state_tok, &code_challenge, &nonce);

    let cookie = header_value(&tx_cookie(sealed).to_string())?;
    Ok((AppendHeaders([(SET_COOKIE, cookie)]), Redirect::to(&url)).into_response())
}

#[derive(Deserialize)]
struct CallbackQuery {
    code: String,
    state: String,
}

async fn auth_callback(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<CallbackQuery>,
) -> Result<Response, AppError> {
    let sealed = read_cookie(&headers, TX_COOKIE)
        .ok_or_else(|| AppError::BadRequest("missing tx cookie".into()))?;
    let tx: TxPayload = open(
        &state.config.auth.session_key,
        TX_COOKIE.as_bytes(),
        &sealed,
    )
    .map_err(|_| AppError::BadRequest("invalid tx cookie".into()))?;
    if !ct_eq(tx.state.as_bytes(), q.state.as_bytes()) {
        return Err(AppError::BadRequest("state mismatch".into()));
    }

    let tokens = state
        .oidc
        .exchange_code(&q.code, &tx.code_verifier)
        .await
        .map_err(AppError::Internal)?;
    let id_token = tokens
        .id_token
        .as_deref()
        .ok_or_else(|| AppError::Internal(anyhow!("token response missing id_token")))?;
    let claims = state
        .oidc
        .validate_id_token(id_token, &tx.nonce)
        .await
        .map_err(AppError::Internal)?;

    let username = claims
        .preferred_username
        .clone()
        .unwrap_or_else(|| claims.sub.clone());
    let session = SessionPayload {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        access_exp: now_secs() + tokens.expires_in,
        username,
        sub: claims.sub,
    };
    let sealed_session = seal(
        &state.config.auth.session_key,
        SESSION_COOKIE.as_bytes(),
        &session,
    )?;

    let session_h = header_value(&session_cookie(sealed_session).to_string())?;
    let clear_tx_h = header_value(&clear_tx_cookie().to_string())?;
    Ok((
        AppendHeaders([(SET_COOKIE, session_h), (SET_COOKIE, clear_tx_h)]),
        Redirect::to("/"),
    )
        .into_response())
}

async fn auth_logout() -> Result<Response, AppError> {
    let cleared = header_value(&clear_session_cookie().to_string())?;
    Ok((
        StatusCode::NO_CONTENT,
        AppendHeaders([(SET_COOKIE, cleared)]),
    )
        .into_response())
}

#[derive(Serialize)]
struct MeResponse {
    username: String,
    sub: String,
}

async fn me(auth: AuthUser) -> Json<MeResponse> {
    Json(MeResponse {
        username: auth.username,
        sub: auth.sub,
    })
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

#[derive(Serialize, Deserialize)]
struct SystemPrompt {
    name: String,
    prompt: String,
    suggested_models: Vec<String>,
}

#[derive(Serialize)]
struct PromptsResponse {
    prompts: Vec<SystemPrompt>,
}

async fn list_prompts(
    _auth: AuthUser,
    State(_state): State<Arc<AppState>>,
) -> Result<Json<PromptsResponse>, AppError> {
    let prompts_paths = vec![
        "/usr/local/etc/gamecode-web/prompts.toml",
        "config/prompts.toml",
    ];

    let mut prompts = None;
    for path in prompts_paths {
        if let Ok(content) = fs::read_to_string(path) {
            tracing::info!("Loading prompts from: {}", path);

            #[derive(Deserialize)]
            struct PromptsConfig {
                prompts: Vec<SystemPrompt>,
            }

            match toml::from_str::<PromptsConfig>(&content) {
                Ok(config) => {
                    prompts = Some(config.prompts);
                    break;
                }
                Err(e) => {
                    tracing::warn!("Failed to parse prompts.toml at {}: {}", path, e);
                }
            }
        }
    }

    let prompts = prompts.unwrap_or_else(|| {
        tracing::info!("Using default prompts (no prompts.toml found)");
        get_default_prompts()
    });

    Ok(Json(PromptsResponse { prompts }))
}

fn get_default_prompts() -> Vec<SystemPrompt> {
    vec![
        SystemPrompt {
            name: "General Assistant".to_string(),
            prompt: "You are a helpful AI assistant.".to_string(),
            suggested_models: vec!["qwen3:14b".to_string()],
        },
        SystemPrompt {
            name: "Custom".to_string(),
            prompt: String::new(),
            suggested_models: vec![],
        },
    ]
}

#[derive(Deserialize)]
struct ChatRequestBody {
    provider: String,
    messages: Vec<crate::providers::ChatMessage>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    system_prompt: Option<String>,
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
        system_prompt: req.system_prompt,
        temperature: req.temperature,
        max_tokens: req.max_tokens,
    };

    tracing::info!("Messages: {:?}", req.messages);

    let mut stream = state.providers.chat(&req.provider, chat_request).await?;
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    tokio::spawn(async move {
        while let Some(result) = stream.next().await {
            match result {
                Ok(chunk) => {
                    let event =
                        Event::default().data(serde_json::to_string(&chunk).unwrap_or_default());
                    if tx.send(Ok(event)).is_err() {
                        break;
                    }
                    if chunk.done {
                        break;
                    }
                }
                Err(e) => {
                    let error_event = Event::default().data(
                        serde_json::to_string(&serde_json::json!({
                            "error": e.to_string()
                        }))
                        .unwrap_or_default(),
                    );
                    let _ = tx.send(Ok(error_event));
                    break;
                }
            }
        }
    });

    Ok(Sse::new(UnboundedReceiverStream::new(rx)))
}

fn read_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    for header in headers.get_all(COOKIE).iter() {
        let Ok(text) = header.to_str() else { continue };
        for cookie in Cookie::split_parse(text).flatten() {
            if cookie.name() == name {
                return Some(cookie.value().to_string());
            }
        }
    }
    None
}

fn header_value(s: &str) -> Result<HeaderValue, AppError> {
    HeaderValue::from_str(s).map_err(|_| AppError::Internal(anyhow!("bad header value")))
}

fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
