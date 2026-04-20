use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use wasm_bindgen::JsCast;
mod api;
mod components;
mod markdown;
mod notebook;
mod simple_storage;
mod storage;

use api::{ApiClient, ApiError};
use components::{
    auth::{redirect_to_login, LoginRedirect},
    chat::Chat,
};

#[derive(Clone, Copy, PartialEq)]
enum AuthState {
    Checking,
    Unauthenticated,
    Authenticated,
}

#[component]
fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text="GameCode Chat"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>

        <Router>
            <Routes>
                <Route path="/" view=HomePage/>
                <Route path="/*any" view=NotFound/>
            </Routes>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    let (auth_state, set_auth_state) = create_signal(AuthState::Checking);
    let (username, set_username) = create_signal(String::new());

    create_effect(move |_| {
        spawn_local(async move {
            let client = ApiClient::new();
            match client.me().await {
                Ok(me) => {
                    set_username.set(me.username);
                    set_auth_state.set(AuthState::Authenticated);
                }
                Err(ApiError::Unauthorized) => {
                    set_auth_state.set(AuthState::Unauthenticated);
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("/api/me failed: {e}").into());
                    set_auth_state.set(AuthState::Unauthenticated);
                }
            }
        });
    });

    view! {
        <div class="app-container">
            {move || match auth_state.get() {
                AuthState::Checking => view! {
                    <div class="auth-container"><p>"Loading…"</p></div>
                }.into_view(),
                AuthState::Unauthenticated => view! { <LoginRedirect/> }.into_view(),
                AuthState::Authenticated => {
                    let user_signal = Signal::derive(move || username.get());
                    view! {
                        <Chat
                            user_name=user_signal
                            on_auth_error=move || redirect_to_login()
                            on_logout=move || {
                                spawn_local(async move {
                                    let _ = ApiClient::new().logout().await;
                                    redirect_to_login();
                                });
                            }
                        />
                    }.into_view()
                }
            }}
        </div>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div class="not-found">
            <h2>"404 - Page Not Found"</h2>
            <a href="/">"Go Home"</a>
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();

    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(app_div) = document.get_element_by_id("app") {
                leptos::mount_to(app_div.unchecked_into(), || view! { <App/> });
            }
        }
    }
}
