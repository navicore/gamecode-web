use leptos::*;
use serde::{Deserialize, Serialize};
use gloo_storage::{LocalStorage, Storage};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub token: String,
    pub expires_at: i64,
}

#[component]
pub fn AuthForm<F>(
    on_auth: F,
) -> impl IntoView 
where
    F: Fn(String) + Clone + 'static,
{
    let (password, set_password) = create_signal(String::new());
    let (error, set_error) = create_signal(Option::<String>::None);
    let (loading, set_loading) = create_signal(false);

    let handle_submit = {
        let on_auth = on_auth.clone();
        move |ev: leptos::ev::SubmitEvent| {
            ev.prevent_default();
            
            let password_value = password.get();
            if password_value.is_empty() {
                set_error.set(Some("Password cannot be empty".to_string()));
                return;
            }

            set_loading.set(true);
            set_error.set(None);

            let on_auth = on_auth.clone();
            spawn_local(async move {
                match authenticate(&password_value).await {
                    Ok(token) => {
                        // Store token in local storage
                        let _ = LocalStorage::set("auth_token", &token);
                        on_auth(token.token);
                    }
                    Err(e) => {
                        set_error.set(Some(e));
                    }
                }
                set_loading.set(false);
            });
        }
    };

    view! {
        <div class="auth-container">
            <form on:submit=handle_submit>
                <h2>"GameCode Authentication"</h2>
                
                {move || error.get().map(|e| view! {
                    <div class="error-message">{e}</div>
                })}

                <input
                    type="password"
                    placeholder="Enter password"
                    value=password
                    on:input=move |ev| set_password.set(event_target_value(&ev))
                    disabled=loading
                />

                <button 
                    type="submit" 
                    disabled=loading
                >
                    {move || if loading.get() { "Authenticating..." } else { "Login" }}
                </button>
            </form>
        </div>
    }
}

async fn authenticate(password: &str) -> Result<AuthToken, String> {
    use gloo_net::http::Request;
    
    #[derive(Serialize)]
    struct AuthRequest {
        password: String,
    }

    let request = AuthRequest {
        password: password.to_string(),
    };

    let response = Request::post("/api/auth")
        .json(&request)
        .map_err(|e| format!("Failed to create request: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if response.ok() {
        response
            .json::<AuthToken>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    } else {
        match response.status() {
            401 => Err("Invalid password".to_string()),
            _ => Err(format!("Authentication failed: {}", response.status())),
        }
    }
}

pub fn get_stored_token() -> Option<AuthToken> {
    LocalStorage::get("auth_token").ok()
}

pub fn clear_auth_token() {
    let _ = LocalStorage::delete("auth_token");
}

pub fn is_token_valid(token: &AuthToken) -> bool {
    use chrono::Utc;
    
    let now = Utc::now().timestamp();
    token.expires_at > now
}