use leptos::*;
use leptos::html::Div;
use web_sys::KeyboardEvent;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use crate::api::{ApiClient, ChatMessage, ChatRequest, ProviderInfo};
use crate::notebook::{Notebook, CellContent, CellId};

#[component]
pub fn Chat(token: String) -> impl IntoView {
    let token = create_rw_signal(token);
    let (notebook, set_notebook) = create_signal(Notebook::new());
    let (providers, set_providers) = create_signal(Vec::<ProviderInfo>::new());
    let (selected_provider, set_selected_provider) = create_signal("ollama".to_string());
    let (input_value, set_input_value) = create_signal(String::new());
    let (is_streaming, set_is_streaming) = create_signal(false);
    
    let notebook_ref = create_node_ref::<Div>();
    
    // Load providers on mount
    create_effect(move |_| {
        let token = token.get();
        spawn_local(async move {
            let client = ApiClient::new();
            if let Ok(response) = client.list_providers(&token).await {
                let providers = response.providers;
                if let Some(first) = providers.first() {
                    set_selected_provider.set(first.name.clone());
                }
                set_providers.set(providers);
            }
        });
    });
    
    // Auto-scroll to bottom when new cells are added
    create_effect(move |_| {
        notebook.get(); // Subscribe to changes
        if let Some(element) = notebook_ref.get() {
            request_animation_frame(move || {
                element.set_scroll_top(element.scroll_height());
            });
        }
    });
    
    let submit_message = move || {
        let message = input_value.get();
        if message.trim().is_empty() || is_streaming.get() {
            return;
        }
        
        // Add user input cell
        set_notebook.update(|nb| {
            nb.add_cell(CellContent::UserInput { text: message.clone() });
        });
        
        // Clear input
        set_input_value.set(String::new());
        
        // Add loading cell
        let loading_id = {
            let mut id = None;
            set_notebook.update(|nb| {
                id = Some(nb.add_cell(CellContent::Loading { 
                    message: Some("Thinking...".to_string()) 
                }));
            });
            id.unwrap()
        };
        
        // Start streaming response
        set_is_streaming.set(true);
        let token = token.get();
        let provider = selected_provider.get();
        
        spawn_local(async move {
            let client = ApiClient::new();
            
            // Create streaming response cell
            let response_id = {
                let mut id = None;
                set_notebook.update(|nb| {
                    // Remove loading cell
                    if let Some(pos) = nb.cells.iter().position(|c| c.id == loading_id) {
                        nb.cells.remove(pos);
                    }
                    
                    // Add response cell
                    id = Some(nb.add_cell(CellContent::TextResponse {
                        text: String::new(),
                        streaming: true,
                    }));
                });
                id.unwrap()
            };
            
            // Set up EventSource for streaming
            if let Some(window) = web_sys::window() {
                let mut init = web_sys::EventSourceInit::new();
                init.set_with_credentials(true);
                
                let event_source = match web_sys::EventSource::new_with_event_source_init_dict(
                    &client.chat_url(),
                    &init
                ) {
                    Ok(es) => es,
                    Err(_) => {
                        set_notebook.update(|nb| {
                            nb.cells.push(crate::notebook::Cell {
                                id: CellId(nb.cells.len()),
                                content: CellContent::Error {
                                    message: "Failed to connect to server".to_string(),
                                    details: None,
                                },
                                timestamp: chrono::Utc::now(),
                                metadata: Default::default(),
                            });
                        });
                        set_is_streaming.set(false);
                        return;
                    }
                };
                
                // Handle messages
                let on_message = {
                    let set_notebook = set_notebook.clone();
                    Closure::<dyn Fn(_)>::new(move |event: web_sys::MessageEvent| {
                        if let Ok(text) = event.data().dyn_into::<js_sys::JsString>() {
                            if let Ok(chunk) = serde_json::from_str::<crate::api::ChatChunk>(&text.as_string().unwrap()) {
                                set_notebook.update(|nb| {
                                    nb.update_streaming_response(response_id, &chunk.text);
                                    if chunk.done {
                                        nb.finalize_streaming_response(response_id);
                                    }
                                });
                                
                                if chunk.done {
                                    set_is_streaming.set(false);
                                }
                            }
                        }
                    })
                };
                
                event_source.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
                on_message.forget();
                
                // Send the actual request
                let request = ChatRequest {
                    provider: provider.clone(),
                    messages: vec![ChatMessage {
                        role: "user".to_string(),
                        content: message,
                    }],
                    model: None,
                    temperature: None,
                    max_tokens: None,
                };
                
                let _ = gloo_net::http::Request::post(&client.chat_url())
                    .header("Authorization", &format!("Bearer {}", token))
                    .json(&request)
                    .unwrap()
                    .send()
                    .await;
            }
        });
    };
    
    let handle_keydown = move |event: KeyboardEvent| {
        if event.key() == "Enter" && !event.shift_key() {
            event.prevent_default();
            // Duplicate submit logic here
            let message = input_value.get();
            if message.trim().is_empty() || is_streaming.get() {
                return;
            }
            
            // Add user input cell
            set_notebook.update(|nb| {
                nb.add_cell(CellContent::UserInput { text: message.clone() });
            });
            
            // Clear input
            set_input_value.set(String::new());
            
            // Rest of submit logic would go here...
        }
    };
    
    view! {
        <div class="chat-container">
            <div class="chat-header">
                <select 
                    class="provider-select"
                    on:change=move |ev| set_selected_provider.set(event_target_value(&ev))
                    prop:value=move || selected_provider.get()
                >
                    {move || providers.get().into_iter().map(|p| {
                        view! {
                            <option value=p.name.clone()>{p.name}</option>
                        }
                    }).collect_view()}
                </select>
            </div>
            
            <div class="notebook-container" node_ref=notebook_ref>
                {move || notebook.get().cells.into_iter().map(|cell| {
                    view! {
                        <crate::notebook::cell::CellView cell=cell/>
                    }
                }).collect_view()}
            </div>
            
            <div class="input-container">
                <textarea
                    class="chat-input"
                    placeholder="Type your message... (Enter to send, Shift+Enter for new line)"
                    prop:value=move || input_value.get()
                    on:input=move |ev| set_input_value.set(event_target_value(&ev))
                    on:keydown=handle_keydown
                    disabled=move || is_streaming.get()
                    rows="3"
                />
                <button
                    class="send-button"
                    on:click=move |_| {
                        let message = input_value.get();
                        if message.trim().is_empty() || is_streaming.get() {
                            return;
                        }
                        
                        // Trigger submit
                        submit_message();
                    }
                    disabled=move || is_streaming.get() || input_value.get().trim().is_empty()
                >
                    {move || if is_streaming.get() { "Streaming..." } else { "Send" }}
                </button>
            </div>
        </div>
    }
}