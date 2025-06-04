use anyhow::Result;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    async_trait,
    extract::{FromRequestParts, TypedHeader},
    headers::{authorization::Bearer, Authorization},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

use crate::config::AuthConfig;

static JWT_SECRET: Lazy<String> = Lazy::new(|| {
    std::env::var("JWT_SECRET").unwrap_or_else(|_| "change-me-in-production".to_string())
});

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // Subject (user id, we'll use "user" for single-user)
    pub exp: i64,     // Expiration time
    pub iat: i64,     // Issued at
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthRequest {
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub expires_in: u64,
}

pub struct AuthUser {
    pub claims: Claims,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, &())
            .await
            .map_err(|_| AuthError::MissingToken)?;

        // Decode and validate the token
        let token_data = decode::<Claims>(
            bearer.token(),
            &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AuthError::InvalidToken)?;

        Ok(AuthUser {
            claims: token_data.claims,
        })
    }
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<()> {
    let parsed_hash = PasswordHash::new(password_hash)?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| anyhow::anyhow!("Invalid password"))
}

pub fn generate_token(config: &AuthConfig) -> Result<String> {
    let now = OffsetDateTime::now_utc();
    let exp = now + Duration::hours(config.session_duration_hours as i64);
    
    let claims = Claims {
        sub: "user".to_string(),
        exp: exp.unix_timestamp(),
        iat: now.unix_timestamp(),
    };
    
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )?;
    
    Ok(token)
}

#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    InvalidToken,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::MissingToken => (StatusCode::UNAUTHORIZED, "Missing authorization token"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid authorization token"),
        };

        let body = Json(serde_json::json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}