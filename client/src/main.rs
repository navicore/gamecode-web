use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use wasm_bindgen::JsCast;
use web_sys::{window, Document};

mod api;
mod components;
mod notebook;
mod storage;
mod simple_storage;

use components::{auth::{AuthForm, get_stored_token, is_token_valid, clear_auth_token}, chat::Chat};

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
    web_sys::console::log_1(&"HomePage component rendering...".into());
    
    let (authenticated, set_authenticated) = create_signal(false);
    let (token, set_token) = create_signal(String::new());
    
    // Check for existing token in localStorage
    create_effect(move |_| {
        if let Some(auth_token) = get_stored_token() {
            if is_token_valid(&auth_token) {
                set_token.set(auth_token.token);
                set_authenticated.set(true);
            } else {
                // Token expired, clear it
                clear_auth_token();
            }
        }
    });
    
    view! {
        <div class="app-container">
            <header>
                <h1>"GameCode Chat"</h1>
                {move || if authenticated.get() {
                    view! {
                        <button 
                            class="logout-btn"
                            on:click=move |_| {
                                set_authenticated.set(false);
                                set_token.set(String::new());
                                clear_auth_token();
                            }
                        >
                            "Logout"
                        </button>
                    }.into_view()
                } else {
                    view! { <span></span> }.into_view()
                }}
            </header>
            
            <main>
                {move || if authenticated.get() {
                    view! {
                        <Chat 
                            token=token.get()
                            on_auth_error=move || {
                                // Clear auth state and logout
                                set_authenticated.set(false);
                                set_token.set(String::new());
                                clear_auth_token();
                            }
                        />
                    }.into_view()
                } else {
                    view! {
                        <AuthForm 
                            on_auth=move |token_value| {
                                set_token.set(token_value.clone());
                                set_authenticated.set(true);
                            }
                        />
                    }.into_view()
                }}
            </main>
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
    
    // Initialize tracing for WASM
    tracing_wasm::set_as_global_default();
    
    // Mount the app to the #app div
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(app_div) = document.get_element_by_id("app") {
                leptos::mount_to(
                    app_div.unchecked_into(),
                    || view! { <App/> },
                );
            }
        }
    }
}