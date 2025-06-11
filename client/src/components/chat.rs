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
    let (selected_model, set_selected_model) = create_signal(String::new());
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
                    if let Some(first_model) = first.models.first() {
                        set_selected_model.set(first_model.clone());
                    }
                }
                set_providers.set(providers);
            }
        });
    });
    
    // Update available models when provider changes
    create_effect(move |_| {
        let provider = selected_provider.get();
        let all_providers = providers.get();
        
        if let Some(provider_info) = all_providers.iter().find(|p| p.name == provider) {
            if let Some(first_model) = provider_info.models.first() {
                set_selected_model.set(first_model.clone());
            }
        }
    });
    
    // Helper function to scroll to bottom
    let scroll_to_bottom = move || {
        if let Some(element) = notebook_ref.get() {
            // Use web_sys to ensure proper scrolling
            if let Some(window) = web_sys::window() {
                let el = element.clone();
                let closure = Closure::once(move || {
                    el.set_scroll_top(el.scroll_height());
                });
                let _ = window.request_animation_frame(closure.as_ref().unchecked_ref());
                closure.forget();
            }
        }
    };
    
    // Auto-scroll to bottom when notebook changes
    create_effect(move |_| {
        notebook.get(); // Subscribe to changes
        scroll_to_bottom();
    });
    
    let submit_message = move || {
        web_sys::console::log_1(&"submit_message called".into());
        let message = input_value.get();
        if message.trim().is_empty() || is_streaming.get() {
            web_sys::console::log_1(&"Message empty or already streaming".into());
            return;
        }
        web_sys::console::log_1(&format!("Submitting message: {}", message).into());
        
        // Add user input cell
        set_notebook.update(|nb| {
            nb.add_cell(CellContent::UserInput { text: message.clone() });
        });
        
        // Clear input
        set_input_value.set(String::new());
        
        // Scroll after adding user message
        scroll_to_bottom();
        
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
        
        // Scroll after adding loading cell
        scroll_to_bottom();
        
        // Create streaming response cell BEFORE async block
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
        
        // Scroll after adding response cell
        scroll_to_bottom();
        
        // Start streaming response
        set_is_streaming.set(true);
        let token = token.get();
        let provider = selected_provider.get();
        let model = selected_model.get();
        
        spawn_local(async move {
            web_sys::console::log_1(&"Inside spawn_local".into());
            let client = ApiClient::new();
            
            // Send request and handle streaming response
            let request = ChatRequest {
                provider: provider.clone(),
                messages: vec![ChatMessage {
                    role: "user".to_string(),
                    content: message,
                }],
                model: Some(model),
                temperature: None,
                max_tokens: None,
            };
            
            // Use web-sys to make the request and handle streaming
            web_sys::console::log_1(&"Starting streaming request".into());
            if let Some(window) = web_sys::window() {
                use wasm_bindgen::JsCast;
                use wasm_bindgen_futures::JsFuture;
                use web_sys::{Request, RequestInit, Response, Headers};
                
                // Create request
                let opts = RequestInit::new();
                opts.set_method("POST");
                
                // Set headers
                let headers = Headers::new().unwrap();
                headers.append("Content-Type", "application/json").unwrap();
                headers.append("Authorization", &format!("Bearer {}", token)).unwrap();
                opts.set_headers(&headers);
                
                // Set body
                let body = serde_json::to_string(&request).unwrap();
                opts.set_body(&wasm_bindgen::JsValue::from_str(&body));
                
                let request = match Request::new_with_str_and_init(&client.chat_url(), &opts) {
                    Ok(req) => req,
                    Err(_) => {
                        set_notebook.update(|nb| {
                            nb.cells.push(crate::notebook::Cell {
                                id: CellId(nb.cells.len()),
                                content: CellContent::Error {
                                    message: "Failed to create request".to_string(),
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
                
                // Fetch and handle response
                web_sys::console::log_1(&format!("Fetching from: {}", client.chat_url()).into());
                let promise = window.fetch_with_request(&request);
                match JsFuture::from(promise).await {
                    Ok(resp_value) => {
                        let resp: Response = resp_value.dyn_into().unwrap();
                        
                        if !resp.ok() {
                            set_notebook.update(|nb| {
                                nb.cells.push(crate::notebook::Cell {
                                    id: CellId(nb.cells.len()),
                                    content: CellContent::Error {
                                        message: format!("Server error: {}", resp.status()),
                                        details: None,
                                    },
                                    timestamp: chrono::Utc::now(),
                                    metadata: Default::default(),
                                });
                            });
                            set_is_streaming.set(false);
                            return;
                        }
                        
                        // Get the body as a stream
                        if let Some(body) = resp.body() {
                            use wasm_streams::ReadableStream;
                            use futures::StreamExt;
                            
                            let stream = ReadableStream::from_raw(body);
                            let mut reader = stream.into_stream();
                            
                            let mut buffer = String::new();
                            
                            while let Some(chunk) = reader.next().await {
                                match chunk {
                                    Ok(data) => {
                                        let array = js_sys::Uint8Array::new(&data);
                                        let mut bytes = vec![0u8; array.length() as usize];
                                        array.copy_to(&mut bytes);
                                        
                                        if let Ok(text) = String::from_utf8(bytes) {
                                            buffer.push_str(&text);
                                            
                                            // Process complete SSE events
                                            while let Some(event_end) = buffer.find("\n\n") {
                                                let event = buffer[..event_end].to_string();
                                                buffer.drain(..event_end + 2);
                                                
                                                // Parse SSE event
                                                if let Some(data_line) = event.lines().find(|line| line.starts_with("data: ")) {
                                                    let data = &data_line[6..]; // Skip "data: "
                                                    
                                                    if let Ok(chunk) = serde_json::from_str::<crate::api::ChatChunk>(data) {
                                                        set_notebook.update(|nb| {
                                                            nb.update_streaming_response(response_id, &chunk.text);
                                                            if chunk.done {
                                                                nb.finalize_streaming_response(response_id);
                                                            }
                                                        });
                                                        
                                                        // Scroll during streaming
                                                        scroll_to_bottom();
                                                        
                                                        if chunk.done {
                                                            set_is_streaming.set(false);
                                                            return;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(_) => {
                                        set_notebook.update(|nb| {
                                            nb.cells.push(crate::notebook::Cell {
                                                id: CellId(nb.cells.len()),
                                                content: CellContent::Error {
                                                    message: "Stream read error".to_string(),
                                                    details: None,
                                                },
                                                timestamp: chrono::Utc::now(),
                                                metadata: Default::default(),
                                            });
                                        });
                                        set_is_streaming.set(false);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => {
                        set_notebook.update(|nb| {
                            nb.cells.push(crate::notebook::Cell {
                                id: CellId(nb.cells.len()),
                                content: CellContent::Error {
                                    message: "Network error".to_string(),
                                    details: None,
                                },
                                timestamp: chrono::Utc::now(),
                                metadata: Default::default(),
                            });
                        });
                        set_is_streaming.set(false);
                    }
                }
            }
        });
    };
    
    let handle_keydown = move |event: KeyboardEvent| {
        if event.key() == "Enter" && !event.shift_key() {
            event.prevent_default();
            submit_message();
        }
    };
    
    view! {
        <div class="chat-container">
            <div class="chat-header">
                <div class="model-selectors">
                    <div class="selector-group">
                        <label>Provider:</label>
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
                    
                    <div class="selector-group">
                        <label>Model:</label>
                        <select 
                            class="model-select"
                            on:change=move |ev| set_selected_model.set(event_target_value(&ev))
                            prop:value=move || selected_model.get()
                        >
                            {move || {
                                let current_provider = selected_provider.get();
                                let all_providers = providers.get();
                                
                                if let Some(provider) = all_providers.iter().find(|p| p.name == current_provider) {
                                    provider.models.clone().into_iter().map(|model| {
                                        view! {
                                            <option value=model.clone()>{model}</option>
                                        }
                                    }).collect_view()
                                } else {
                                    vec![].into_view()
                                }
                            }}
                        </select>
                    </div>
                </div>
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