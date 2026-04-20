use std::sync::Arc;

use axum::{
    async_trait,
    body::Body,
    extract::{FromRequestParts, Request, State},
    http::{
        header::{COOKIE, SET_COOKIE},
        request::Parts,
        HeaderValue, StatusCode,
    },
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use cookie::Cookie;
use serde_json::json;

use super::session::{
    clear_session_cookie, open, seal, session_cookie, SessionPayload, SESSION_COOKIE,
};
use crate::AppState;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub username: String,
    pub sub: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .ok_or_else(unauthorized)
    }
}

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    match try_auth(&state, &mut req).await {
        Ok(refreshed_cookie) => {
            let mut resp = next.run(req).await;
            if let Some(cookie) = refreshed_cookie {
                if let Ok(val) = HeaderValue::from_str(&cookie) {
                    resp.headers_mut().append(SET_COOKIE, val);
                }
            }
            resp
        }
        Err(_) => unauthorized(),
    }
}

async fn try_auth(state: &AppState, req: &mut Request<Body>) -> Result<Option<String>, AuthFail> {
    let sealed = read_session_cookie(req).ok_or(AuthFail)?;
    let key = &state.config.auth.session_key;
    let mut session: SessionPayload =
        open(key, SESSION_COOKIE.as_bytes(), &sealed).map_err(|_| AuthFail)?;

    let now = now_secs();
    let mut refreshed = false;
    if session.access_exp <= now {
        let tokens = state
            .oidc
            .refresh(&session.refresh_token)
            .await
            .map_err(|_| AuthFail)?;
        state
            .oidc
            .validate_access_token(&tokens.access_token)
            .await
            .map_err(|_| AuthFail)?;
        session.access_token = tokens.access_token;
        session.refresh_token = tokens.refresh_token;
        session.access_exp = now + tokens.expires_in;
        refreshed = true;
    } else {
        state
            .oidc
            .validate_access_token(&session.access_token)
            .await
            .map_err(|_| AuthFail)?;
    }

    req.extensions_mut().insert(AuthUser {
        username: session.username.clone(),
        sub: session.sub.clone(),
    });

    if refreshed {
        let sealed = seal(key, SESSION_COOKIE.as_bytes(), &session).map_err(|_| AuthFail)?;
        Ok(Some(session_cookie(sealed).to_string()))
    } else {
        Ok(None)
    }
}

fn read_session_cookie(req: &Request<Body>) -> Option<String> {
    for header in req.headers().get_all(COOKIE).iter() {
        let Ok(text) = header.to_str() else { continue };
        for cookie in Cookie::split_parse(text).flatten() {
            if cookie.name() == SESSION_COOKIE {
                return Some(cookie.value().to_string());
            }
        }
    }
    None
}

fn now_secs() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

struct AuthFail;

fn unauthorized() -> Response {
    let cookie = clear_session_cookie().to_string();
    let mut resp = (
        StatusCode::UNAUTHORIZED,
        Json(json!({ "error": "Unauthorized" })),
    )
        .into_response();
    if let Ok(val) = HeaderValue::from_str(&cookie) {
        resp.headers_mut().append(SET_COOKIE, val);
    }
    resp
}
