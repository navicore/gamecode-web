use leptos::*;

pub const LOGIN_PATH: &str = "/api/auth/login";

pub fn redirect_to_login() {
    if let Some(win) = web_sys::window() {
        let _ = win.location().assign(LOGIN_PATH);
    }
}

#[component]
pub fn LoginRedirect() -> impl IntoView {
    create_effect(|_| {
        redirect_to_login();
    });
    view! {
        <div class="auth-container">
            <p>"Redirecting to login…"</p>
        </div>
    }
}
