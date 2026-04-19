use crate::api::{ApiClient, ChatMessage, ChatRequest, ProviderInfo, SystemPrompt};
use crate::components::composer::Composer;
use crate::components::context_manager::ContextManager;
use crate::components::empty_state::EmptyState;
use crate::components::model_picker::ModelPicker;
use crate::components::persona_picker::PersonaPicker;
use crate::components::sidebar::Sidebar;
use crate::components::sidebar_resize::{load_saved_width, SidebarResize};
use crate::notebook::cell::{CellContext, CellView};
use crate::notebook::{CellContent, CellId, Notebook};
use crate::simple_storage::SimpleStorage;
use crate::storage::{ConversationMetadata, StoredConversation};
use chrono::Utc;
use leptos::html::Div;
use leptos::*;
use uuid::Uuid;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

fn user_name_from_token(token: &str) -> String {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return "friend".into();
    }
    let payload_b64 = parts[1];
    let padded = match payload_b64.len() % 4 {
        0 => payload_b64.to_string(),
        r => format!("{}{}", payload_b64, "=".repeat(4 - r)),
    };
    let bytes = match base64_url_decode(&padded) {
        Some(b) => b,
        None => return "friend".into(),
    };
    let s = match std::str::from_utf8(&bytes) {
        Ok(s) => s,
        Err(_) => return "friend".into(),
    };
    serde_json::from_str::<serde_json::Value>(s)
        .ok()
        .and_then(|v| v.get("sub").and_then(|x| x.as_str()).map(|x| x.to_string()))
        .unwrap_or_else(|| "friend".into())
}

fn base64_url_decode(s: &str) -> Option<Vec<u8>> {
    let std = s.replace('-', "+").replace('_', "/");
    base64_decode(&std)
}

fn base64_decode(s: &str) -> Option<Vec<u8>> {
    const TBL: [i8; 128] = {
        let mut t = [-1i8; 128];
        let mut i = 0u8;
        while i < 26 {
            t[(b'A' + i) as usize] = i as i8;
            t[(b'a' + i) as usize] = (i + 26) as i8;
            i += 1;
        }
        let mut j = 0u8;
        while j < 10 {
            t[(b'0' + j) as usize] = (j + 52) as i8;
            j += 1;
        }
        t[b'+' as usize] = 62;
        t[b'/' as usize] = 63;
        t
    };
    let mut out = Vec::with_capacity(s.len() * 3 / 4);
    let bytes = s.as_bytes();
    let mut buf: u32 = 0;
    let mut bits = 0u32;
    for &b in bytes {
        if b == b'=' {
            break;
        }
        if (b as usize) >= TBL.len() {
            return None;
        }
        let v = TBL[b as usize];
        if v < 0 {
            return None;
        }
        buf = (buf << 6) | (v as u32);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((buf >> bits) & 0xFF) as u8);
        }
    }
    Some(out)
}

fn read_local(key: &str) -> Option<String> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(key).ok().flatten())
}

fn write_local(key: &str, value: &str) {
    if let Some(s) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = s.set_item(key, value);
    }
}

#[component]
pub fn Chat<F, G>(token: String, on_auth_error: F, on_logout: G) -> impl IntoView
where
    F: Fn() + Clone + 'static,
    G: Fn() + Clone + 'static,
{
    let user_label = user_name_from_token(&token);
    let token = create_rw_signal(token);
    let (notebook, set_notebook) = create_signal(Notebook::new());
    let (auth_error_triggered, set_auth_error_triggered) = create_signal(false);

    let saved_provider = read_local("selected_provider").unwrap_or_default();
    let saved_model = read_local("selected_model").unwrap_or_default();
    let saved_prompt =
        read_local("selected_prompt").unwrap_or_else(|| "General Assistant".to_string());
    let saved_custom = read_local("custom_prompt").unwrap_or_default();
    let saved_input = read_local("pending_input").unwrap_or_default();
    let saved_temp = read_local("temperature")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.7);
    let saved_theme = read_local("gc_theme").unwrap_or_else(|| "light".to_string());

    let providers = create_rw_signal(Vec::<ProviderInfo>::new());
    let selected_provider = create_rw_signal(saved_provider);
    let selected_model = create_rw_signal(saved_model);
    let system_prompts = create_rw_signal(Vec::<SystemPrompt>::new());
    let selected_prompt_name = create_rw_signal(saved_prompt);
    let custom_prompt = create_rw_signal(saved_custom);
    let temperature = create_rw_signal(saved_temp);
    let input_value = create_rw_signal(saved_input.clone());
    let (is_streaming, set_is_streaming) = create_signal(false);
    let (should_submit, set_should_submit) = create_signal(false);
    let (providers_loaded, set_providers_loaded) = create_signal(false);
    let (initial_load_complete, set_initial_load_complete) = create_signal(false);
    let sidebar_width = create_rw_signal(load_saved_width());
    let theme = create_rw_signal(saved_theme);
    let search_query = create_rw_signal(String::new());

    if !saved_input.is_empty() {
        if let Some(s) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
            let _ = s.remove_item("pending_input");
        }
    }

    let thread_ref = create_node_ref::<Div>();

    let (conversation_id, set_conversation_id) =
        create_signal(read_local("current_conversation_id").unwrap_or_else(|| {
            let new_id = Uuid::new_v4().to_string();
            write_local("current_conversation_id", &new_id);
            new_id
        }));

    let context_manager = ContextManager::new();
    let simple_storage = SimpleStorage::new();
    let (created_at, set_created_at) = create_signal(Utc::now());
    let (conversations, set_conversations) =
        create_signal(Vec::<crate::storage::ConversationRef>::new());

    create_effect(move |_| {
        if auth_error_triggered.get() {
            on_auth_error();
        }
    });

    create_effect({
        let context_manager = context_manager.clone();
        let simple_storage = simple_storage.clone();
        move |_| {
            let current_id = conversation_id.get();
            if let Ok(Some(stored)) = simple_storage.load_conversation(&current_id) {
                context_manager.restore_state(stored.context_state);
                set_notebook.update(|nb| *nb = stored.notebook);
                set_created_at.set(stored.metadata.created_at);
            }
        }
    });

    create_effect({
        let simple_storage = simple_storage.clone();
        move |_| {
            if let Ok(list) = simple_storage.list_conversations(50) {
                set_conversations.set(list);
            }
        }
    });

    // Load providers and prompts on mount
    create_effect(move |_| {
        let tk = token.get();
        spawn_local(async move {
            let client = ApiClient::new();
            match client.list_providers(&tk).await {
                Ok(resp) => {
                    let list = resp.providers;
                    providers.set(list.clone());
                    if !initial_load_complete.get_untracked() {
                        let cp = selected_provider.get_untracked();
                        let cm = selected_model.get_untracked();
                        if cp.is_empty() && cm.is_empty() {
                            if let Some(first) = list.first() {
                                selected_provider.set(first.name.clone());
                                if let Some(m) = first.models.first() {
                                    selected_model.set(m.clone());
                                }
                            }
                        }
                        set_initial_load_complete.set(true);
                    }
                    set_providers_loaded.set(true);
                }
                Err(crate::api::ApiError::Unauthorized) => set_auth_error_triggered.set(true),
                Err(e) => {
                    web_sys::console::error_1(&format!("providers: {}", e).into());
                }
            }
            match client.list_prompts(&tk).await {
                Ok(resp) => system_prompts.set(resp.prompts),
                Err(crate::api::ApiError::Unauthorized) => set_auth_error_triggered.set(true),
                Err(e) => {
                    web_sys::console::error_1(&format!("prompts: {}", e).into());
                }
            }
        });
    });

    // Persist selections
    create_effect(move |_| {
        let v = selected_provider.get();
        if initial_load_complete.get() && !v.is_empty() {
            write_local("selected_provider", &v);
        }
    });
    create_effect(move |_| {
        let v = selected_model.get();
        if initial_load_complete.get() && !v.is_empty() {
            write_local("selected_model", &v);
        }
    });
    create_effect(move |_| {
        let v = selected_prompt_name.get();
        if initial_load_complete.get() {
            write_local("selected_prompt", &v);
        }
    });
    create_effect(move |_| {
        let v = custom_prompt.get();
        if initial_load_complete.get() {
            write_local("custom_prompt", &v);
        }
    });
    create_effect(move |_| {
        let v = temperature.get();
        if initial_load_complete.get() {
            write_local("temperature", &v.to_string());
        }
    });

    // When a new provider is selected, if current model isn't in its list, pick first
    create_effect(move |_| {
        let p = selected_provider.get();
        let all = providers.get();
        if let Some(info) = all.iter().find(|x| x.name == p) {
            let cm = selected_model.get_untracked();
            if !info.models.iter().any(|m| m == &cm) {
                if let Some(m) = info.models.first() {
                    selected_model.set(m.clone());
                }
            }
        }
    });

    let scroll_to_bottom = move || {
        if let Some(el) = thread_ref.get_untracked() {
            if let Some(win) = web_sys::window() {
                let el2 = el.clone();
                let closure = Closure::once(move || el2.set_scroll_top(el2.scroll_height()));
                let _ = win.request_animation_frame(closure.as_ref().unchecked_ref());
                closure.forget();
            }
        }
    };

    create_effect(move |_| {
        let _ = notebook.get();
        scroll_to_bottom();
    });

    // Save conversation when notebook changes
    create_effect({
        let context_manager = context_manager.clone();
        let simple_storage = simple_storage.clone();
        move |_| {
            let nb = notebook.get();
            let _ = context_manager.get_total_tokens();
            let title = nb
                .cells
                .iter()
                .find_map(|c| match &c.content {
                    CellContent::UserInput { text } => Some(text.clone()),
                    _ => None,
                })
                .map(|t| {
                    let line = t.lines().next().unwrap_or(&t);
                    let truncated = line.chars().take(40).collect::<String>();
                    if line.len() > 40 {
                        format!("{}...", truncated)
                    } else {
                        truncated
                    }
                })
                .unwrap_or_else(|| "new".into());

            let metadata = ConversationMetadata {
                created_at: created_at.get(),
                modified_at: Utc::now(),
                title,
                model: selected_model.get(),
                provider: selected_provider.get(),
            };
            let stored = StoredConversation {
                id: conversation_id.get(),
                notebook: nb,
                context_state: context_manager.to_state(),
                metadata,
            };
            if simple_storage.save_conversation(&stored).is_ok() {
                if let Ok(list) = simple_storage.list_conversations(50) {
                    set_conversations.set(list);
                }
            }
        }
    });

    create_effect({
        let context_manager = context_manager.clone();
        move |_| {
            if !should_submit.get() {
                return;
            }
            set_should_submit.set(false);
            let message = input_value.get();
            if message.trim().is_empty() || is_streaming.get_untracked() {
                return;
            }
            set_notebook.update(|nb| {
                nb.add_cell(CellContent::UserInput {
                    text: message.clone(),
                });
            });
            context_manager.add_message(ChatMessage {
                role: "user".into(),
                content: message.clone(),
            });
            input_value.set(String::new());

            let response_id = {
                let mut id = None;
                set_notebook.update(|nb| {
                    id = Some(nb.add_cell(CellContent::TextResponse {
                        text: String::new(),
                        streaming: true,
                    }));
                });
                id.unwrap()
            };

            set_is_streaming.set(true);
            let tk = token.get_untracked();
            let provider = selected_provider.get_untracked();
            let model = selected_model.get_untracked();
            let prompt_name = selected_prompt_name.get_untracked();
            let custom = custom_prompt.get_untracked();
            let prompts_snapshot = system_prompts.get_untracked();
            let system_prompt = if prompt_name == "Custom" {
                Some(custom)
            } else {
                prompts_snapshot
                    .iter()
                    .find(|p| p.name == prompt_name)
                    .map(|p| p.prompt.clone())
            };
            let cm_clone = context_manager.clone();
            spawn_local(async move {
                stream_response(
                    tk,
                    provider,
                    model,
                    system_prompt,
                    temperature.get_untracked(),
                    cm_clone.clone(),
                    set_notebook,
                    response_id,
                    set_is_streaming,
                    set_auth_error_triggered,
                    message,
                )
                .await;
            });
        }
    });

    let has_messages = create_memo(move |_| {
        notebook.get().cells.iter().any(|c| {
            matches!(
                &c.content,
                CellContent::UserInput { .. } | CellContent::TextResponse { .. }
            )
        })
    });

    let provider_online = Signal::derive(move || !providers.get().is_empty());
    let user_signal = Signal::derive({
        let label = user_label.clone();
        move || label.clone()
    });

    let simple_storage_new = simple_storage.clone();
    let cm_for_new = context_manager.clone();
    let on_new_chat = Callback::new(move |_| {
        let new_id = Uuid::new_v4().to_string();
        write_local("current_conversation_id", &new_id);
        set_conversation_id.set(new_id);
        set_notebook.update(|nb| {
            nb.cells.clear();
            nb.cursor_position = CellId(0);
        });
        cm_for_new.clear_context();
        set_created_at.set(Utc::now());
        if let Ok(list) = simple_storage_new.list_conversations(50) {
            set_conversations.set(list);
        }
    });

    let simple_storage_sel = simple_storage.clone();
    let cm_for_sel = context_manager.clone();
    let on_select = Callback::new(move |id: String| {
        if id == conversation_id.get_untracked() {
            return;
        }
        write_local("current_conversation_id", &id);
        set_conversation_id.set(id.clone());
        if let Ok(Some(stored)) = simple_storage_sel.load_conversation(&id) {
            cm_for_sel.restore_state(stored.context_state);
            set_notebook.update(|nb| *nb = stored.notebook);
            set_created_at.set(stored.metadata.created_at);
        }
    });

    let simple_storage_del = simple_storage.clone();
    let cm_for_del = context_manager.clone();
    let on_delete = Callback::new(move |id: String| {
        let _ = simple_storage_del.delete_conversation(&id);
        if let Ok(list) = simple_storage_del.list_conversations(50) {
            set_conversations.set(list);
        }
        if id == conversation_id.get_untracked() {
            let new_id = Uuid::new_v4().to_string();
            write_local("current_conversation_id", &new_id);
            set_conversation_id.set(new_id);
            set_notebook.update(|nb| {
                nb.cells.clear();
                nb.cursor_position = CellId(0);
            });
            cm_for_del.clear_context();
            set_created_at.set(Utc::now());
        }
    });

    let logout_cb = on_logout.clone();
    let on_logout_cb = Callback::new(move |_| logout_cb());

    let on_submit = Callback::new(move |_| set_should_submit.set(true));

    let on_pick_suggestion = Callback::new(move |text: String| {
        input_value.set(text);
    });

    let user_initial = user_label
        .chars()
        .next()
        .map(|c| c.to_ascii_uppercase().to_string())
        .unwrap_or_else(|| "U".into());

    let cm_for_composer = context_manager.clone();

    view! {
        <div class="app" style:grid-template-columns=move || format!("{}px 1fr", sidebar_width.get())>
            <Sidebar
                conversations=conversations.into()
                current_id=conversation_id
                search=search_query
                theme=theme
                provider_online=provider_online
                user_name=user_signal
                on_new=on_new_chat
                on_select=on_select
                on_delete=on_delete
                on_logout=on_logout_cb
            />
            <SidebarResize width=sidebar_width/>
            <main class="main">
                <div class="chat-header">
                    <div class="chat-title">
                        <span class="chat-title-text">
                            {move || {
                                let id = conversation_id.get();
                                conversations.get().iter()
                                    .find(|c| c.id == id)
                                    .map(|c| c.title.clone())
                                    .unwrap_or_else(|| "New chat".into())
                            }}
                        </span>
                    </div>
                    {move || if providers_loaded.get() {
                        view! {
                            <>
                                <ModelPicker
                                    providers=providers.read_only()
                                    selected_provider=selected_provider
                                    selected_model=selected_model
                                    disabled=has_messages.into()
                                />
                                <PersonaPicker
                                    prompts=system_prompts.read_only()
                                    selected_name=selected_prompt_name
                                    custom_prompt=custom_prompt
                                />
                            </>
                        }.into_view()
                    } else {
                        view! { <span class="chat-title-meta">"Loading…"</span> }.into_view()
                    }}
                </div>

                <div class="thread-wrap" node_ref=thread_ref>
                    {move || if !has_messages.get() {
                        view! {
                            <EmptyState
                                user_name=user_signal
                                on_pick=on_pick_suggestion
                            />
                        }.into_view()
                    } else {
                        let initial_for_each = user_initial.clone();
                        view! {
                            <div class="thread">
                                <For
                                    each=move || notebook.get().cells
                                    key=|c| c.id.0
                                    children=move |cell| {
                                        let ctx = CellContext {
                                            user_initial: initial_for_each.clone(),
                                            persona_name: selected_prompt_name.get_untracked(),
                                        };
                                        view! { <CellView cell=cell ctx=ctx notebook=notebook/> }
                                    }
                                />
                            </div>
                        }.into_view()
                    }}
                </div>
                <Composer
                    input_value=input_value
                    is_streaming=is_streaming
                    temperature=temperature
                    context_manager=cm_for_composer
                    on_submit=on_submit
                />
            </main>
        </div>
    }
}

#[allow(clippy::too_many_arguments)]
async fn stream_response(
    token: String,
    provider: String,
    model: String,
    system_prompt: Option<String>,
    temperature: f32,
    context_manager: ContextManager,
    set_notebook: WriteSignal<Notebook>,
    response_id: CellId,
    set_is_streaming: WriteSignal<bool>,
    set_auth_error: WriteSignal<bool>,
    pending_message: String,
) {
    use futures::FutureExt;
    use futures::StreamExt;
    use gloo_timers::future::TimeoutFuture;
    use wasm_bindgen_futures::JsFuture;
    use wasm_streams::ReadableStream;
    use web_sys::{Headers, Request, RequestInit, Response};

    let client = ApiClient::new();
    let context_messages = context_manager.get_context_for_request();
    let req = ChatRequest {
        provider,
        messages: context_messages,
        model: Some(model),
        system_prompt,
        temperature: Some(temperature),
        max_tokens: None,
    };

    let push_error = move |msg: &str, details: Option<String>| {
        set_notebook.update(|nb| {
            nb.cells.push(crate::notebook::Cell {
                id: CellId(nb.cells.len()),
                content: CellContent::Error {
                    message: msg.to_string(),
                    details,
                },
                timestamp: chrono::Utc::now(),
                metadata: Default::default(),
            });
        });
    };

    let timeout = TimeoutFuture::new(120_000);
    let request_future = async {
        let window = match web_sys::window() {
            Some(w) => w,
            None => {
                push_error("No window context", None);
                set_is_streaming.set(false);
                return;
            }
        };

        let opts = RequestInit::new();
        opts.set_method("POST");
        let headers = Headers::new().unwrap();
        headers.append("Content-Type", "application/json").unwrap();
        headers
            .append("Authorization", &format!("Bearer {}", token))
            .unwrap();
        opts.set_headers(&headers);
        let body = serde_json::to_string(&req).unwrap();
        opts.set_body(&wasm_bindgen::JsValue::from_str(&body));

        let request = match Request::new_with_str_and_init(&client.chat_url(), &opts) {
            Ok(r) => r,
            Err(_) => {
                push_error("Failed to create request", None);
                set_is_streaming.set(false);
                return;
            }
        };

        let resp: Response = match JsFuture::from(window.fetch_with_request(&request)).await {
            Ok(v) => v.dyn_into().unwrap(),
            Err(_) => {
                push_error("Network error", None);
                set_is_streaming.set(false);
                return;
            }
        };

        if !resp.ok() {
            if resp.status() == 401 {
                write_local("pending_input", &pending_message);
                push_error(
                    "Authentication expired. Please log in again.",
                    Some("Your message has been saved and will be restored after login.".into()),
                );
                set_is_streaming.set(false);
                spawn_local(async move {
                    gloo_timers::future::sleep(std::time::Duration::from_secs(2)).await;
                    set_auth_error.set(true);
                });
                return;
            }
            push_error(&format!("Server error: {}", resp.status()), None);
            set_is_streaming.set(false);
            return;
        }

        let body = match resp.body() {
            Some(b) => b,
            None => {
                push_error("Empty response", None);
                set_is_streaming.set(false);
                return;
            }
        };

        let mut reader = ReadableStream::from_raw(body).into_stream();
        let mut buffer = String::new();
        let mut full = String::new();
        while let Some(chunk) = reader.next().await {
            match chunk {
                Ok(data) => {
                    let arr = js_sys::Uint8Array::new(&data);
                    let mut bytes = vec![0u8; arr.length() as usize];
                    arr.copy_to(&mut bytes);
                    let Ok(text) = String::from_utf8(bytes) else {
                        continue;
                    };
                    buffer.push_str(&text);
                    while let Some(end) = buffer.find("\n\n") {
                        let event = buffer[..end].to_string();
                        buffer.drain(..end + 2);
                        let Some(data_line) = event.lines().find(|l| l.starts_with("data: "))
                        else {
                            continue;
                        };
                        let Ok(chunk) =
                            serde_json::from_str::<crate::api::ChatChunk>(&data_line[6..])
                        else {
                            continue;
                        };
                        full.push_str(&chunk.text);
                        set_notebook.update(|nb| {
                            nb.update_streaming_response(response_id, &chunk.text);
                            if chunk.done {
                                nb.finalize_streaming_response(response_id);
                            }
                        });
                        if chunk.done {
                            context_manager.add_message(ChatMessage {
                                role: "assistant".into(),
                                content: full.clone(),
                            });
                            set_is_streaming.set(false);
                            return;
                        }
                    }
                }
                Err(_) => {
                    push_error("Stream read error", None);
                    set_is_streaming.set(false);
                    return;
                }
            }
        }
    };

    futures::select! {
        _ = request_future.fuse() => {}
        _ = timeout.fuse() => {
            set_is_streaming.set(false);
            set_notebook.update(|nb| {
                if let Some(pos) = nb.cells.iter().position(|c| c.id == response_id) {
                    nb.cells.remove(pos);
                }
                nb.cells.push(crate::notebook::Cell {
                    id: CellId(nb.cells.len()),
                    content: CellContent::Error {
                        message: "Request timed out".into(),
                        details: Some("The model didn't respond within 2 minutes.".into()),
                    },
                    timestamp: chrono::Utc::now(),
                    metadata: Default::default(),
                });
            });
        }
    }
}
