use crate::components::icons::*;
use crate::storage::ConversationRef;
use chrono::{DateTime, Local, TimeZone, Utc};
use leptos::*;
use wasm_bindgen::JsCast;

fn theme_next(current: &str) -> &'static str {
    if current == "dark" {
        "light"
    } else {
        "dark"
    }
}

fn apply_theme_attr(theme: &str) {
    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
        if let Some(root) = doc.document_element() {
            if let Ok(html) = root.dyn_into::<web_sys::HtmlElement>() {
                let _ = html.dataset().set("theme", theme);
            }
        }
    }
}

fn bucket(dt: &DateTime<Utc>) -> &'static str {
    let local = Local.from_utc_datetime(&dt.naive_utc());
    let days = Local::now().signed_duration_since(local).num_days();
    if days <= 0 {
        "Today"
    } else if days == 1 {
        "Yesterday"
    } else {
        "Earlier"
    }
}

fn short_time(dt: &DateTime<Utc>) -> String {
    let local = Local.from_utc_datetime(&dt.naive_utc());
    let days = Local::now().signed_duration_since(local).num_days();
    if days <= 0 {
        local.format("%H:%M").to_string()
    } else if days < 7 {
        local.format("%a").to_string()
    } else {
        local.format("%b %d").to_string()
    }
}

#[component]
pub fn Sidebar(
    conversations: Signal<Vec<ConversationRef>>,
    current_id: ReadSignal<String>,
    search: RwSignal<String>,
    theme: RwSignal<String>,
    provider_online: Signal<bool>,
    user_name: Signal<String>,
    on_new: Callback<()>,
    on_select: Callback<String>,
    on_delete: Callback<String>,
    on_logout: Callback<()>,
) -> impl IntoView {
    let filtered = move || {
        let q = search.get().trim().to_lowercase();
        conversations
            .get()
            .into_iter()
            .filter(|c| {
                q.is_empty()
                    || c.title.to_lowercase().contains(&q)
                    || c.preview.to_lowercase().contains(&q)
            })
            .collect::<Vec<_>>()
    };

    let grouped = move || {
        let mut today = Vec::new();
        let mut yesterday = Vec::new();
        let mut earlier = Vec::new();
        for c in filtered() {
            match bucket(&c.modified_at) {
                "Today" => today.push(c),
                "Yesterday" => yesterday.push(c),
                _ => earlier.push(c),
            }
        }
        vec![
            ("Today", today),
            ("Yesterday", yesterday),
            ("Earlier", earlier),
        ]
    };

    let toggle_theme = move |_| {
        let next = theme_next(&theme.get_untracked()).to_string();
        apply_theme_attr(&next);
        if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
            let _ = storage.set_item("gc_theme", &next);
        }
        theme.set(next);
    };

    view! {
        <aside class="sidebar">
            <div class="sidebar-head">
                <div class="brand">
                    <div class="brand-mark">"gc"</div>
                    <span>"Gamecode"</span>
                </div>
                <button class="icon-btn" title="Toggle theme" on:click=toggle_theme>
                    {move || if theme.get() == "dark" {
                        view! { <IconSun/> }.into_view()
                    } else {
                        view! { <IconMoon/> }.into_view()
                    }}
                </button>
            </div>

            <button class="new-chat" on:click=move |_| on_new.call(())>
                <span class="new-chat-inner">
                    <IconPlus/>
                    <span>"New chat"</span>
                </span>
                <span class="new-chat-kbd">"⌘ N"</span>
            </button>

            <div class="search">
                <IconSearch/>
                <input
                    type="text"
                    placeholder="Search chats"
                    prop:value=move || search.get()
                    on:input=move |ev| search.set(event_target_value(&ev))
                />
            </div>

            <div class="conv-list">
                {move || grouped().into_iter().filter(|(_, items)| !items.is_empty()).map(|(label, items)| {
                    view! {
                        <>
                            <div class="conv-section-label">{label}</div>
                            {items.into_iter().map(|c| {
                                let cid = c.id.clone();
                                let cid_sel = c.id.clone();
                                let cid_del = c.id.clone();
                                let is_active = create_memo(move |_| current_id.get() == cid);
                                let title = c.title.clone();
                                let meta = short_time(&c.modified_at);
                                view! {
                                    <div class="conv" class:active=move || is_active.get()>
                                        <span class="conv-dot"></span>
                                        <span
                                            class="conv-title"
                                            on:click=move |_| on_select.call(cid_sel.clone())
                                        >{title}</span>
                                        <span class="conv-meta">{meta}</span>
                                        <button
                                            class="conv-del"
                                            title="Delete"
                                            on:click=move |e| {
                                                e.stop_propagation();
                                                on_delete.call(cid_del.clone());
                                            }
                                        >
                                            <IconTrash/>
                                        </button>
                                    </div>
                                }
                            }).collect_view()}
                        </>
                    }
                }).collect_view()}
            </div>

            <div class="sidebar-foot">
                <div class="user-chip">
                    <div class="avatar">
                        {move || user_name.get().chars().next()
                            .map(|c| c.to_ascii_uppercase().to_string())
                            .unwrap_or_else(|| "·".to_string())}
                    </div>
                    <div class="user-meta">
                        <div class="user-name">{move || user_name.get()}</div>
                        <div class="user-status">
                            <span
                                class="status-dot"
                                class:offline=move || !provider_online.get()
                            ></span>
                            <span>
                                {move || if provider_online.get() { "Ollama" } else { "Offline" }}
                            </span>
                        </div>
                    </div>
                </div>
                <button class="icon-btn" title="Sign out" on:click=move |_| on_logout.call(())>
                    <IconLogout/>
                </button>
            </div>
        </aside>
    }
}
