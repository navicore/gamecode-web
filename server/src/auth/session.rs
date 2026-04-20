use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce,
};
use anyhow::{anyhow, bail, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD as B64, Engine as _};
use cookie::{time::Duration, Cookie, SameSite};
use rand::RngCore;
use serde::{Deserialize, Serialize};

pub const SESSION_COOKIE: &str = "__Host-gc_session";
pub const TX_COOKIE: &str = "__Host-gc_oidc_tx";

const SESSION_MAX_AGE: Duration = Duration::days(14);
const TX_MAX_AGE: Duration = Duration::minutes(5);
const NONCE_LEN: usize = 12;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionPayload {
    pub access_token: String,
    pub refresh_token: String,
    pub access_exp: i64,
    pub username: String,
    pub sub: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxPayload {
    pub code_verifier: String,
    pub state: String,
    pub nonce: String,
}

pub fn seal<T: Serialize>(key: &[u8; 32], aad: &[u8], value: &T) -> Result<String> {
    let plaintext = serde_json::to_vec(value).context("serialize cookie payload")?;
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| anyhow!("bad key length"))?;
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::rng().fill_bytes(&mut nonce_bytes);
    let ciphertext = cipher
        .encrypt(
            Nonce::from_slice(&nonce_bytes),
            Payload {
                msg: &plaintext,
                aad,
            },
        )
        .map_err(|_| anyhow!("AEAD encrypt failed"))?;
    let mut buf = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    buf.extend_from_slice(&nonce_bytes);
    buf.extend_from_slice(&ciphertext);
    Ok(B64.encode(buf))
}

pub fn open<T: for<'de> Deserialize<'de>>(key: &[u8; 32], aad: &[u8], sealed: &str) -> Result<T> {
    let bytes = B64.decode(sealed).context("decode cookie base64")?;
    if bytes.len() <= NONCE_LEN {
        bail!("cookie too short");
    }
    let (nonce, ct) = bytes.split_at(NONCE_LEN);
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| anyhow!("bad key length"))?;
    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce), Payload { msg: ct, aad })
        .map_err(|_| anyhow!("AEAD decrypt failed"))?;
    serde_json::from_slice(&plaintext).context("deserialize cookie payload")
}

pub fn session_cookie(value: String) -> Cookie<'static> {
    build_cookie(SESSION_COOKIE, value, SESSION_MAX_AGE)
}

pub fn tx_cookie(value: String) -> Cookie<'static> {
    build_cookie(TX_COOKIE, value, TX_MAX_AGE)
}

pub fn clear_session_cookie() -> Cookie<'static> {
    cleared(SESSION_COOKIE)
}

pub fn clear_tx_cookie() -> Cookie<'static> {
    cleared(TX_COOKIE)
}

fn build_cookie(name: &'static str, value: String, max_age: Duration) -> Cookie<'static> {
    Cookie::build((name, value))
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .max_age(max_age)
        .build()
}

fn cleared(name: &'static str) -> Cookie<'static> {
    Cookie::build((name, String::new()))
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .max_age(Duration::ZERO)
        .build()
}
