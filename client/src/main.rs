use leptos::*;
use leptos_meta::*;
use leptos_router::*;

mod api;
mod components;
mod notebook;

use components::{auth::AuthForm, chat::Chat};

#[component]
fn App() -> impl IntoView {
    provide_meta_context();
    
    view! {
        <Stylesheet id="leptos" href="/style.css"/>
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
    let (authenticated, set_authenticated) = create_signal(false);
    let (token, set_token) = create_signal(String::new());
    
    // Check for existing token in localStorage
    create_effect(move |_| {
        if let Ok(Some(stored_token)) = window().local_storage() {
            if let Ok(Some(t)) = stored_token.get_item("auth_token") {
                if !t.is_empty() {
                    set_token.set(t);
                    set_authenticated.set(true);
                }
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
                                if let Ok(Some(storage)) = window().local_storage() {
                                    let _ = storage.remove_item("auth_token");
                                }
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
                        <Chat token=token.get()/>
                    }.into_view()
                } else {
                    view! {
                        <AuthForm 
                            on_auth=move |token_value| {
                                set_token.set(token_value.clone());
                                set_authenticated.set(true);
                                // Store token
                                if let Ok(Some(storage)) = window().local_storage() {
                                    let _ = storage.set_item("auth_token", &token_value);
                                }
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
    
    mount_to_body(|| view! { <App/> })
}