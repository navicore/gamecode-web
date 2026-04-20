use anyhow::{anyhow, bail, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD as B64, Engine as _};
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation};
use rand::RngCore;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

use crate::config::OidcConfig;

const JWKS_REFRESH_COOLDOWN: Duration = Duration::from_secs(60);

pub struct OidcClient {
    pub config: OidcConfig,
    http: reqwest::Client,
    metadata: Metadata,
    jwks: Arc<RwLock<JwksCache>>,
}

#[derive(Debug, Clone, Deserialize)]
struct Metadata {
    authorization_endpoint: String,
    token_endpoint: String,
    jwks_uri: String,
    issuer: String,
}

struct JwksCache {
    keys: HashMap<String, DecodingKey>,
    last_fetch: Option<Instant>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: String,
    #[serde(default)]
    pub id_token: Option<String>,
    pub expires_in: i64,
}

#[derive(Debug, Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}

#[derive(Debug, Deserialize)]
struct Jwk {
    kid: String,
    kty: String,
    n: Option<String>,
    e: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct IdTokenClaims {
    pub sub: String,
    #[serde(default)]
    pub preferred_username: Option<String>,
    pub nonce: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AccessClaims {
    pub exp: i64,
}

impl OidcClient {
    pub async fn discover(config: OidcConfig) -> Result<Self> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;
        let discovery_url = format!("{}/.well-known/openid-configuration", config.issuer);
        let metadata: Metadata = http
            .get(&discovery_url)
            .send()
            .await
            .with_context(|| format!("GET {discovery_url}"))?
            .error_for_status()?
            .json()
            .await?;
        if metadata.issuer != config.issuer {
            bail!(
                "issuer mismatch: discovered={}, configured={}",
                metadata.issuer,
                config.issuer
            );
        }
        let jwks = Arc::new(RwLock::new(JwksCache {
            keys: HashMap::new(),
            last_fetch: None,
        }));
        let client = Self {
            config,
            http,
            metadata,
            jwks,
        };
        client.refresh_jwks(true).await?;
        Ok(client)
    }

    pub fn authorize_url(&self, state: &str, code_challenge: &str, nonce: &str) -> String {
        let params = [
            ("response_type", "code"),
            ("client_id", &self.config.client_id),
            ("redirect_uri", &self.config.redirect_uri),
            ("scope", &self.config.scopes),
            ("state", state),
            ("code_challenge", code_challenge),
            ("code_challenge_method", "S256"),
            ("nonce", nonce),
        ];
        let qs = serde_urlencoded::to_string(params).unwrap_or_default();
        format!("{}?{qs}", self.metadata.authorization_endpoint)
    }

    pub async fn exchange_code(&self, code: &str, code_verifier: &str) -> Result<Tokens> {
        let form = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &self.config.redirect_uri),
            ("code_verifier", code_verifier),
        ];
        self.token_request(&form).await
    }

    pub async fn refresh(&self, refresh_token: &str) -> Result<Tokens> {
        let form = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
        ];
        self.token_request(&form).await
    }

    async fn token_request(&self, form: &[(&str, &str)]) -> Result<Tokens> {
        let resp = self
            .http
            .post(&self.metadata.token_endpoint)
            .basic_auth(&self.config.client_id, Some(&self.config.client_secret))
            .form(form)
            .send()
            .await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            bail!("token endpoint {status}: {body}");
        }
        Ok(resp.json().await?)
    }

    pub async fn validate_id_token(&self, id_token: &str, nonce: &str) -> Result<IdTokenClaims> {
        let header = decode_header(id_token)?;
        let kid = header
            .kid
            .clone()
            .ok_or_else(|| anyhow!("id token: no kid"))?;
        let key = self.key_for(&kid).await?;
        let mut v = Validation::new(header.alg);
        v.set_issuer(&[&self.config.issuer]);
        v.set_audience(&[&self.config.client_id]);
        v.validate_exp = true;
        let data = decode::<IdTokenClaims>(id_token, &key, &v)?;
        if data.claims.nonce.as_deref() != Some(nonce) {
            bail!("id token nonce mismatch");
        }
        Ok(data.claims)
    }

    pub async fn validate_access_token(&self, access_token: &str) -> Result<AccessClaims> {
        let header = decode_header(access_token)?;
        let kid = header
            .kid
            .clone()
            .ok_or_else(|| anyhow!("access token: no kid"))?;
        let key = self.key_for(&kid).await?;
        let mut v = Validation::new(header.alg);
        v.set_issuer(&[&self.config.issuer]);
        v.validate_aud = false;
        v.validate_exp = true;
        Ok(decode::<AccessClaims>(access_token, &key, &v)?.claims)
    }

    async fn key_for(&self, kid: &str) -> Result<DecodingKey> {
        if let Some(k) = self.jwks.read().await.keys.get(kid).cloned() {
            return Ok(k);
        }
        self.refresh_jwks(false).await?;
        self.jwks
            .read()
            .await
            .keys
            .get(kid)
            .cloned()
            .ok_or_else(|| anyhow!("unknown jwks kid: {kid}"))
    }

    async fn refresh_jwks(&self, force: bool) -> Result<()> {
        {
            let guard = self.jwks.read().await;
            if !force {
                if let Some(when) = guard.last_fetch {
                    if when.elapsed() < JWKS_REFRESH_COOLDOWN {
                        return Ok(());
                    }
                }
            }
        }
        let jwks: Jwks = self
            .http
            .get(&self.metadata.jwks_uri)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        let mut map = HashMap::with_capacity(jwks.keys.len());
        for jwk in jwks.keys {
            if jwk.kty != "RSA" {
                continue;
            }
            let (Some(n), Some(e)) = (jwk.n.as_deref(), jwk.e.as_deref()) else {
                continue;
            };
            if let Ok(key) = DecodingKey::from_rsa_components(n, e) {
                map.insert(jwk.kid, key);
            }
        }
        let mut guard = self.jwks.write().await;
        guard.keys = map;
        guard.last_fetch = Some(Instant::now());
        Ok(())
    }
}

pub fn random_b64_url(bytes: usize) -> String {
    let mut buf = vec![0u8; bytes];
    rand::rng().fill_bytes(&mut buf);
    B64.encode(buf)
}

pub fn pkce_challenge(verifier: &str) -> String {
    let digest = Sha256::digest(verifier.as_bytes());
    B64.encode(digest)
}
