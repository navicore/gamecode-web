use leptos::*;

const SUGGESTIONS: &[(&str, &str)] = &[
    (
        "Plan",
        "Outline a migration from REST to gRPC for a small service",
    ),
    (
        "Debug",
        "Help me reason about a deadlock in an async Rust program",
    ),
    ("Review", "Review this SQL schema for normalization issues"),
    (
        "Explain",
        "Explain how SSE streaming differs from WebSockets",
    ),
];

#[component]
pub fn EmptyState(user_name: Signal<String>, on_pick: Callback<String>) -> impl IntoView {
    view! {
        <div class="empty">
            <h1>{move || format!("Good to see you, {}.", user_name.get())}</h1>
            <p>"What are we working on today?"</p>
            <div class="suggestions">
                {SUGGESTIONS.iter().map(|(label, text)| {
                    let text = text.to_string();
                    let text_clone = text.clone();
                    view! {
                        <button
                            class="suggestion"
                            on:click=move |_| on_pick.call(text_clone.clone())
                        >
                            <div class="suggestion-label">{*label}</div>
                            <div class="suggestion-text">{text}</div>
                        </button>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}
