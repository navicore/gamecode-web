use leptos::*;
use leptos::html::Div;
use web_sys::KeyboardEvent;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use crate::api::{ApiClient, ChatMessage, ChatRequest, ProviderInfo, SystemPrompt};
use crate::notebook::{Notebook, CellContent, CellId};
use crate::components::context_manager::{ContextManager, ContextDisplay};
use crate::storage::{StoredConversation, ConversationMetadata};
use crate::simple_storage::SimpleStorage;
use chrono::{Utc, Local, TimeZone};
use uuid::Uuid;

#[component]
pub fn Chat<F>(
    token: String,
    on_auth_error: F,
) -> impl IntoView 
where
    F: Fn() + Clone + 'static,
{
    let token = create_rw_signal(token);
    let (notebook, set_notebook) = create_signal(Notebook::new());
    let (auth_error_triggered, set_auth_error_triggered) = create_signal(false);
    
    // Load saved preferences from localStorage
    let window = web_sys::window().unwrap();
    let storage = window.local_storage().unwrap().unwrap();
    
    let saved_provider = storage.get_item("selected_provider").ok().flatten().unwrap_or_default();
    let saved_model = storage.get_item("selected_model").ok().flatten().unwrap_or_default();
    let saved_prompt = storage.get_item("selected_prompt").ok().flatten().unwrap_or_else(|| "General Assistant".to_string());
    let saved_custom = storage.get_item("custom_prompt").ok().flatten().unwrap_or_default();
    let saved_input = storage.get_item("pending_input").ok().flatten().unwrap_or_default();
    let saved_temperature = storage.get_item("temperature").ok().flatten()
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.7);
    
    // Debug log
    web_sys::console::log_1(&format!("Component init - localStorage - Provider: '{}', Model: '{}', Prompt: '{}'", 
        &saved_provider, &saved_model, &saved_prompt).into());
    
    let (providers, set_providers) = create_signal(Vec::<ProviderInfo>::new());
    let (selected_provider, set_selected_provider) = create_signal(saved_provider);
    let (selected_model, set_selected_model) = create_signal(saved_model);
    let (system_prompts, set_system_prompts) = create_signal(Vec::<SystemPrompt>::new());
    let (selected_prompt_name, set_selected_prompt_name) = create_signal(saved_prompt);
    let (custom_prompt, set_custom_prompt) = create_signal(saved_custom);
    let (temperature, set_temperature) = create_signal(saved_temperature);
    let (input_value, set_input_value) = create_signal(saved_input.clone());
    let (is_streaming, set_is_streaming) = create_signal(false);
    let (should_submit, set_should_submit) = create_signal(false);
    let (provider_manually_changed, set_provider_manually_changed) = create_signal(false);
    let (providers_loaded, set_providers_loaded) = create_signal(false);
    let (initial_load_complete, set_initial_load_complete) = create_signal(false);
    let (has_messages, set_has_messages) = create_signal(false);
    
    // Clear saved input after loading
    if !saved_input.is_empty() {
        let _ = storage.remove_item("pending_input");
    }
    
    let notebook_ref = create_node_ref::<Div>();
    
    // Create or restore conversation ID
    let (conversation_id, set_conversation_id) = create_signal(
        storage.get_item("current_conversation_id")
            .ok()
            .flatten()
            .unwrap_or_else(|| {
                let new_id = Uuid::new_v4().to_string();
                let _ = storage.set_item("current_conversation_id", &new_id);
                new_id
            })
    );
    
    // Create context manager
    let context_manager = ContextManager::new();
    let simple_storage = SimpleStorage::new();
    let (created_at, set_created_at) = create_signal(Utc::now());
    let (conversations, set_conversations) = create_signal(Vec::<crate::storage::ConversationRef>::new());
    let (show_conversation_dropdown, set_show_conversation_dropdown) = create_signal(false);
    
    // Handle auth errors
    create_effect(move |_| {
        if auth_error_triggered.get() {
            on_auth_error();
        }
    });
    
    // Setup click outside handler for dropdown
    create_effect(move |_| {
        if show_conversation_dropdown.get() {
            // Add a small delay to prevent immediate closing when opening
            if let Some(window) = web_sys::window() {
                let closure = Closure::once(move || {
                    if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                        let handle_click = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
                            if let Some(target) = e.target() {
                                if let Ok(element) = target.dyn_into::<web_sys::Element>() {
                                    // Check if click is outside the dropdown area
                                    if element.closest(".dropdown-wrapper").unwrap_or(None).is_none() {
                                        set_show_conversation_dropdown.set(false);
                                    }
                                }
                            }
                        }) as Box<dyn FnMut(_)>);
                        
                        let _ = document.add_event_listener_with_callback(
                            "click",
                            handle_click.as_ref().unchecked_ref()
                        );
                        handle_click.forget();
                    }
                });
                
                let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    10
                );
                closure.forget();
            }
        }
    });
    
    // Load existing conversation on mount
    create_effect({
        let conversation_id = conversation_id.clone();
        let context_manager = context_manager.clone();
        let simple_storage = simple_storage.clone();
        move |_| {
            let current_id = conversation_id.get();
            web_sys::console::log_1(&"Loading conversation from localStorage...".into());
            
            // Try to load existing conversation
            if let Ok(Some(stored)) = simple_storage.load_conversation(&current_id) {
                web_sys::console::log_1(&format!("Loaded conversation {} with {} cells", 
                    current_id, stored.notebook.cells.len()).into());
                    
                // Restore the context state
                context_manager.restore_state(stored.context_state);
                
                // Check if conversation has messages before moving notebook
                let has_msgs = stored.notebook.cells.iter().any(|cell| matches!(
                    &cell.content, 
                    CellContent::UserInput { .. } | CellContent::TextResponse { .. }
                ));
                set_has_messages.set(has_msgs);
                
                // Update notebook with stored cells
                set_notebook.update(|nb| {
                    *nb = stored.notebook;
                });
                
                // Restore created time
                set_created_at.set(stored.metadata.created_at);
                
                web_sys::console::log_1(&"Conversation restored successfully".into());
            } else {
                web_sys::console::log_1(&format!("No existing conversation found for ID: {}", current_id).into());
            }
        }
    });
    
    // Load conversations list
    create_effect({
        let simple_storage = simple_storage.clone();
        move |_| {
            if let Ok(conv_list) = simple_storage.list_conversations(20) {
                set_conversations.set(conv_list);
            }
        }
    });
    
    // Load providers and prompts on mount
    create_effect(move |_| {
        let token = token.get();
        spawn_local(async move {
            let client = ApiClient::new();
            
            // Load providers
            match client.list_providers(&token).await {
                Ok(response) => {
                    let providers = response.providers;
                    set_providers.set(providers.clone());
                    
                    if !initial_load_complete.get() {
                        let current_provider = selected_provider.get();
                        let current_model = selected_model.get();
                        
                        web_sys::console::log_1(&format!("Providers loaded. Current signals - Provider: '{}', Model: '{}'", 
                            &current_provider, &current_model).into());
                        
                        // Only set defaults if we have absolutely nothing saved
                        if current_provider.is_empty() && current_model.is_empty() {
                            // No saved values at all, use first provider
                            if let Some(first) = providers.first() {
                                web_sys::console::log_1(&"No saved values, setting first provider as default".into());
                                set_selected_provider.set(first.name.clone());
                                if let Some(first_model) = first.models.first() {
                                    set_selected_model.set(first_model.clone());
                                }
                            }
                        }
                        // If we have a saved provider/model, trust it and don't change anything
                        // The UI will handle invalid selections
                        
                        set_initial_load_complete.set(true);
                    }
                    
                    set_providers_loaded.set(true);
                }
                Err(crate::api::ApiError::Unauthorized) => {
                    set_auth_error_triggered.set(true);
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to load providers: {}", e).into());
                }
            }
            
            // Load prompts
            match client.list_prompts(&token).await {
                Ok(response) => {
                    set_system_prompts.set(response.prompts);
                }
                Err(crate::api::ApiError::Unauthorized) => {
                    set_auth_error_triggered.set(true);
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to load prompts: {}", e).into());
                }
            }
        });
    });
    
    // Save provider selection when it changes
    create_effect(move |_| {
        let provider = selected_provider.get();
        // Only save after initial load is complete to avoid overwriting saved values
        if initial_load_complete.get() && !provider.is_empty() {
            web_sys::console::log_1(&format!("Saving provider to localStorage: {}", &provider).into());
            if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
                let _ = storage.set_item("selected_provider", &provider);
            }
        }
    });
    
    // Update model when provider is manually changed
    create_effect(move |_| {
        if provider_manually_changed.get() {
            let provider = selected_provider.get();
            let all_providers = providers.get();
            
            if let Some(provider_info) = all_providers.iter().find(|p| p.name == provider) {
                if let Some(first_model) = provider_info.models.first() {
                    set_selected_model.set(first_model.clone());
                }
            }
            set_provider_manually_changed.set(false);
        }
    });
    
    // Update system prompt when model changes
    create_effect(move |_| {
        let model = selected_model.get();
        let prompts = system_prompts.get();
        
        // Save to localStorage (only after initial load is complete)
        if !model.is_empty() && initial_load_complete.get() {
            web_sys::console::log_1(&format!("Saving model to localStorage: {}", &model).into());
            if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
                let _ = storage.set_item("selected_model", &model);
            }
        }
        
        // Only auto-select prompt if user hasn't already chosen one
        if selected_prompt_name.get() == "General Assistant" {
            // Find a prompt that suggests this model
            let matching_prompt = prompts.iter()
                .find(|p| p.suggested_models.iter().any(|m| m.contains(&model)))
                .or_else(|| prompts.iter().find(|p| p.name == "General Assistant"));
                
            if let Some(prompt) = matching_prompt {
                set_selected_prompt_name.set(prompt.name.clone());
            }
        }
    });
    
    // Save prompt selection changes
    create_effect(move |_| {
        let prompt_name = selected_prompt_name.get();
        if initial_load_complete.get() {
            web_sys::console::log_1(&format!("Saving prompt to localStorage: {}", &prompt_name).into());
            if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
                let _ = storage.set_item("selected_prompt", &prompt_name);
            }
        }
    });
    
    // Save custom prompt changes
    create_effect(move |_| {
        let custom = custom_prompt.get();
        if initial_load_complete.get() {
            if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
                let _ = storage.set_item("custom_prompt", &custom);
            }
        }
    });
    
    // Save temperature changes
    create_effect(move |_| {
        let temp = temperature.get();
        if initial_load_complete.get() {
            if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
                let _ = storage.set_item("temperature", &temp.to_string());
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
        let nb = notebook.get(); // Subscribe to changes
        
        // Update has_messages based on notebook content
        let has_msgs = nb.cells.iter().any(|cell| matches!(
            &cell.content, 
            CellContent::UserInput { .. } | CellContent::TextResponse { .. }
        ));
        set_has_messages.set(has_msgs);
        
        scroll_to_bottom();
    });
    
    // Save conversation when notebook or context changes
    create_effect({
        let conversation_id = conversation_id.clone();
        let context_manager = context_manager.clone();
        let selected_provider = selected_provider.clone();
        let selected_model = selected_model.clone();
        let simple_storage = simple_storage.clone();
        move |_| {
            let notebook_value = notebook.get();
            
            // Also subscribe to context changes
            let _ = context_manager.get_total_tokens();
            
            // Skip if no content
            if notebook_value.cells.is_empty() {
                return;
            }
            
            web_sys::console::log_1(&format!("Save effect - preparing to save {} cells", notebook_value.cells.len()).into());
            
            // Generate title from first user message
            let title = notebook_value.cells.iter()
                .find_map(|cell| match &cell.content {
                    crate::notebook::CellContent::UserInput { text } => Some(text.clone()),
                    _ => None
                })
                .map(|text| {
                    // Take first 40 chars or up to first newline
                    let title_text = text.lines().next().unwrap_or(&text);
                    let truncated = title_text.chars().take(40).collect::<String>();
                    if title_text.len() > 40 {
                        truncated + "..."
                    } else {
                        truncated
                    }
                })
                .unwrap_or_else(|| "New Conversation".to_string());
            
            // Create metadata
            let metadata = ConversationMetadata {
                created_at: created_at.get(),
                modified_at: Utc::now(),
                title,
                model: selected_model.get(),
                provider: selected_provider.get(),
            };
            
            // Create stored conversation
            let stored = StoredConversation {
                id: conversation_id.get(),
                notebook: notebook_value,
                context_state: context_manager.to_state(),
                metadata,
            };
            
            web_sys::console::log_1(&format!("Saving conversation {}", conversation_id.get()).into());
            
            // Save synchronously
            match simple_storage.save_conversation(&stored) {
                Ok(_) => {
                    web_sys::console::log_1(&format!("✅ Conversation {} saved successfully", stored.id).into());
                    // Refresh conversation list
                    if let Ok(conv_list) = simple_storage.list_conversations(20) {
                        set_conversations.set(conv_list);
                    }
                },
                Err(e) => web_sys::console::error_1(&format!("❌ Failed to save conversation: {:?}", e).into()),
            }
        }
    });
    
    // Handle message submission when triggered
    create_effect({
        let context_manager = context_manager.clone();
        move |_| {
            if !should_submit.get() {
                return;
            }
            
            // Reset the trigger
            set_should_submit.set(false);
            
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
        
        // Add message to context
        let user_msg = ChatMessage {
            role: "user".to_string(),
            content: message.clone(),
        };
        context_manager.add_message(user_msg);
        
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
        let prompt_name = selected_prompt_name.get();
        let custom = custom_prompt.get();
        let prompts = system_prompts.get();
        
        // Get the actual system prompt text
        let system_prompt = if prompt_name == "Custom" {
            Some(custom)
        } else {
            prompts
                .iter()
                .find(|p| p.name == prompt_name)
                .map(|p| p.prompt.clone())
        };
        
        let context_manager_clone = context_manager.clone();
        spawn_local(async move {
            web_sys::console::log_1(&"Inside spawn_local".into());
            let client = ApiClient::new();
            
            // Get full context for request
            let context_messages = context_manager_clone.get_context_for_request();
            
            // Send request and handle streaming response
            let request = ChatRequest {
                provider: provider.clone(),
                messages: context_messages,
                model: Some(model),
                system_prompt,
                temperature: Some(temperature.get_untracked()),
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
                            if resp.status() == 401 {
                                // Save the current input before logging out
                                if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
                                    let _ = storage.set_item("pending_input", &message);
                                }
                                
                                set_notebook.update(|nb| {
                                    nb.cells.push(crate::notebook::Cell {
                                        id: CellId(nb.cells.len()),
                                        content: CellContent::Error {
                                            message: "Authentication expired. Please log in again.".to_string(),
                                            details: Some("Your message has been saved and will be restored after login.".to_string()),
                                        },
                                        timestamp: chrono::Utc::now(),
                                        metadata: Default::default(),
                                    });
                                });
                                set_is_streaming.set(false);
                                
                                // Trigger auth error callback after a short delay to show the message
                                spawn_local(async move {
                                    gloo_timers::future::sleep(std::time::Duration::from_secs(2)).await;
                                    set_auth_error_triggered.set(true);
                                });
                                return;
                            }
                            
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
                            let mut complete_response = String::new();
                            
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
                                                        complete_response.push_str(&chunk.text);
                                                        
                                                        set_notebook.update(|nb| {
                                                            nb.update_streaming_response(response_id, &chunk.text);
                                                            if chunk.done {
                                                                nb.finalize_streaming_response(response_id);
                                                            }
                                                        });
                                                        
                                                        // Scroll during streaming
                                                        scroll_to_bottom();
                                                        
                                                        if chunk.done {
                                                            // Add complete response to context
                                                            let assistant_msg = ChatMessage {
                                                                role: "assistant".to_string(),
                                                                content: complete_response.clone(),
                                                            };
                                                            context_manager_clone.add_message(assistant_msg);
                                                            
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
        }
    });
    
    let handle_keydown = move |event: KeyboardEvent| {
        if event.key() == "Enter" && !event.shift_key() {
            event.prevent_default();
            set_should_submit.set(true);
        }
    };
    
    let context_manager_for_display = context_manager.clone();
    let context_manager_for_clear = context_manager.clone();
    
    view! {
        <div class="chat-container">
            <div class="chat-header">
                <div class="selectors-container">
                    <div class="selector-column">
                        <label class="selector-label">Conversations</label>
                        <div class="dropdown-wrapper">
                            <button 
                                class="selector-dropdown conversation-button"
                                on:click=move |e| {
                                    e.stop_propagation();
                                    set_show_conversation_dropdown.update(|v| *v = !*v);
                                }
                            >
                                {move || {
                                    let current_id = conversation_id.get();
                                    conversations.get().iter()
                                        .find(|c| c.id == current_id)
                                        .map(|c| c.title.clone())
                                        .unwrap_or_else(|| "New Conversation".to_string())
                                }}
                                <span class="dropdown-arrow">{"▼"}</span>
                            </button>
                            
                            {move || if show_conversation_dropdown.get() {
                                let context_manager_clone = context_manager.clone();
                                let conversation_id = conversation_id.clone();
                                let simple_storage = simple_storage.clone();
                                let context_manager = context_manager.clone();
                                view! {
                                    <div class="conversation-dropdown-menu">
                                        <button 
                                            class="new-conversation-btn"
                                            on:click=move |_| {
                                                // Create new conversation
                                                let new_id = Uuid::new_v4().to_string();
                                                if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
                                                    let _ = storage.set_item("current_conversation_id", &new_id);
                                                }
                                                set_conversation_id.set(new_id);
                                                set_notebook.update(|nb| {
                                                    nb.cells.clear();
                                                    nb.cursor_position = crate::notebook::CellId(0);
                                                });
                                                context_manager_clone.clear_context();
                                                set_created_at.set(Utc::now());
                                                set_has_messages.set(false);
                                                set_show_conversation_dropdown.set(false);
                                            }
                                        >
                                            "+ New Conversation"
                                        </button>
                                        
                                        <div class="conversation-count">
                                            {move || format!("{} / 25 conversations", conversations.get().len())}
                                        </div>
                                        
                                        <div class="conversation-list">
                                            {move || conversations.get().into_iter().map({
                                                let conversation_id = conversation_id.clone();
                                                let simple_storage = simple_storage.clone();
                                                let context_manager = context_manager.clone();
                                                move |conv| {
                                                    let is_current = conv.id == conversation_id.get();
                                                    let conv_id = conv.id.clone();
                                                    let conv_clone = conv.clone();
                                                    view! {
                                                        <div class="conversation-item" class:current=is_current>
                                                            <button
                                                                class="conversation-select-btn"
                                                                on:click={
                                                                    let conv_id = conv_id.clone();
                                                                    let simple_storage = simple_storage.clone();
                                                                    let context_manager = context_manager.clone();
                                                                    move |_| {
                                                                        if !is_current {
                                                                            // Switch to this conversation
                                                                            if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
                                                                                let _ = storage.set_item("current_conversation_id", &conv_id);
                                                                            }
                                                                            set_conversation_id.set(conv_id.clone());
                                                                            set_show_conversation_dropdown.set(false);
                                                                            
                                                                            // Load the conversation
                                                                            if let Ok(Some(stored)) = simple_storage.load_conversation(&conv_id) {
                                                                                context_manager.restore_state(stored.context_state);
                                                                                
                                                                                // Check if conversation has messages
                                                                                let has_msgs = stored.notebook.cells.iter().any(|cell| matches!(
                                                                                    &cell.content, 
                                                                                    CellContent::UserInput { .. } | CellContent::TextResponse { .. }
                                                                                ));
                                                                                set_has_messages.set(has_msgs);
                                                                                
                                                                                set_notebook.update(|nb| {
                                                                                    *nb = stored.notebook;
                                                                                });
                                                                                set_created_at.set(stored.metadata.created_at);
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            >
                                                                <div class="conversation-title">{conv_clone.title}</div>
                                                                <div class="conversation-preview">{conv_clone.preview}</div>
                                                                <div class="conversation-date">{format_conversation_date(&conv_clone.modified_at)}</div>
                                                            </button>
                                                            <button
                                                                class="conversation-delete-btn"
                                                                on:click={
                                                                    let conv_id = conv.id.clone();
                                                                    let simple_storage = simple_storage.clone();
                                                                    let conversation_id_clone = conversation_id.clone();
                                                                    let context_manager_clone = context_manager.clone();
                                                                    move |e| {
                                                                        e.stop_propagation();
                                                                        // Delete the conversation
                                                                        let _ = simple_storage.delete_conversation(&conv_id);
                                                                        // Refresh list
                                                                        if let Ok(conv_list) = simple_storage.list_conversations(20) {
                                                                            set_conversations.set(conv_list);
                                                                        }
                                                                        // If deleting current conversation, create new one
                                                                        if conv_id == conversation_id_clone.get() {
                                                                            let new_id = Uuid::new_v4().to_string();
                                                                            if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
                                                                                let _ = storage.set_item("current_conversation_id", &new_id);
                                                                            }
                                                                            set_conversation_id.set(new_id);
                                                                            set_notebook.update(|nb| {
                                                                                nb.cells.clear();
                                                                                nb.cursor_position = crate::notebook::CellId(0);
                                                                            });
                                                                            context_manager_clone.clear_context();
                                                                            set_created_at.set(Utc::now());
                                                                            set_has_messages.set(false);
                                                                        }
                                                                    }
                                                                }
                                                                title="Delete conversation"
                                                            >
                                                                "×"
                                                            </button>
                                                        </div>
                                                    }
                                                }
                                            }).collect_view()}
                                        </div>
                                    </div>
                                }.into_view()
                            } else {
                                view! { <div></div> }.into_view()
                            }}
                        </div>
                    </div>
                    
                    {move || if providers_loaded.get() {
                        view! {
                            <>
                                <div class="selector-column">
                                    <label class="selector-label">Provider</label>
                                    <select 
                                        class="selector-dropdown"
                                        disabled=move || has_messages.get()
                                        on:change=move |ev| {
                                            set_provider_manually_changed.set(true);
                                            set_selected_provider.set(event_target_value(&ev));
                                        }
                                    >
                                        {move || {
                                            let current = selected_provider.get();
                                            providers.get().into_iter().map(|p| {
                                                let is_selected = p.name == current;
                                                view! {
                                                    <option value=p.name.clone() selected=is_selected>{p.name}</option>
                                                }
                                            }).collect_view()
                                        }}
                                    </select>
                                </div>
                                
                                <div class="selector-column">
                                    <label class="selector-label">Model</label>
                                    <select 
                                        class="selector-dropdown"
                                        disabled=move || has_messages.get()
                                        on:change=move |ev| set_selected_model.set(event_target_value(&ev))
                                    >
                                        {move || {
                                            let current_provider = selected_provider.get();
                                            let current_model = selected_model.get();
                                            let all_providers = providers.get();
                                            
                                            if let Some(provider) = all_providers.iter().find(|p| p.name == current_provider) {
                                                provider.models.clone().into_iter().map(|model| {
                                                    let is_selected = model == current_model;
                                                    view! {
                                                        <option value=model.clone() selected=is_selected>{model}</option>
                                                    }
                                                }).collect_view()
                                            } else {
                                                leptos::View::default()
                                            }
                                        }}
                                    </select>
                                </div>
                                
                                <div class="selector-column">
                                    <label class="selector-label">Persona</label>
                                    <select 
                                        class="selector-dropdown"
                                        on:change=move |ev| set_selected_prompt_name.set(event_target_value(&ev))
                                    >
                                        {move || {
                                            let current = selected_prompt_name.get();
                                            system_prompts.get().into_iter().map(|prompt| {
                                                let is_selected = prompt.name == current;
                                                view! {
                                                    <option value=prompt.name.clone() selected=is_selected>{prompt.name}</option>
                                                }
                                            }).collect_view()
                                        }}
                                    </select>
                                </div>
                                
                                <div class="selector-column">
                                    <label class="selector-label">Temperature: {move || format!("{:.1}", temperature.get())}</label>
                                    <input 
                                        type="range"
                                        class="temperature-slider"
                                        min="0.0"
                                        max="1.0"
                                        step="0.1"
                                        prop:value=move || temperature.get().to_string()
                                        on:input=move |ev| {
                                            if let Ok(val) = event_target_value(&ev).parse::<f32>() {
                                                set_temperature.set(val);
                                            }
                                        }
                                    />
                                </div>
                            </>
                        }.into_view()
                    } else {
                        view! {
                            <div class="loading-message">Loading providers...</div>
                        }.into_view()
                    }}
                </div>
                
                {move || {
                    if selected_prompt_name.get() == "Custom" {
                        view! {
                            <div class="custom-prompt-container">
                                <textarea
                                    class="custom-prompt-input"
                                    placeholder="Enter your custom system prompt..."
                                    prop:value=move || custom_prompt.get()
                                    on:input=move |ev| set_custom_prompt.set(event_target_value(&ev))
                                    rows="3"
                                />
                            </div>
                        }.into_view()
                    } else {
                        view! { <div></div> }.into_view()
                    }
                }}
            </div>
            
            <div class="notebook-container" node_ref=notebook_ref>
                {move || notebook.get().cells.into_iter().map(|cell| {
                    view! {
                        <crate::notebook::cell::CellView cell=cell/>
                    }
                }).collect_view()}
            </div>
            
            <div class="input-container">
                <div class="input-wrapper" class=("streaming", move || is_streaming.get())>
                    {move || if is_streaming.get() {
                        view! {
                            <div class="streaming-indicator">
                                <div class="streaming-dots">
                                    <span class="dot"></span>
                                    <span class="dot"></span>
                                    <span class="dot"></span>
                                </div>
                                <span class="streaming-text">"Generating"</span>
                            </div>
                        }.into_view()
                    } else {
                        view! { <span></span> }.into_view()
                    }}
                    <textarea
                        class="chat-input"
                        placeholder="Type your message... (Enter to send, Shift+Enter for new line)"
                        prop:value=move || input_value.get()
                        on:input=move |ev| set_input_value.set(event_target_value(&ev))
                        on:keydown=handle_keydown
                        disabled=move || is_streaming.get()
                        rows="3"
                    />
                </div>
                <ContextDisplay 
                    context_manager=context_manager_for_display
                    on_compress=move || {
                        web_sys::console::log_1(&"Compress context clicked".into());
                        // TODO: Implement compression
                    }
                    on_clear={
                        move || {
                            web_sys::console::log_1(&"Clear context clicked".into());
                            context_manager_for_clear.clear_context();
                            set_notebook.update(|nb| {
                                nb.cells.clear();
                                nb.cursor_position = CellId(0);
                            });
                            set_has_messages.set(false);
                            
                            // Generate new conversation ID
                            let new_id = Uuid::new_v4().to_string();
                            if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
                                let _ = storage.set_item("current_conversation_id", &new_id);
                            }
                            
                            // Reload to start fresh
                            let _ = web_sys::window().unwrap().location().reload();
                        }
                    }
                />
            </div>
        </div>
    }
}

fn format_conversation_date(dt: &chrono::DateTime<chrono::Utc>) -> String {
    // Convert UTC to local time
    let local_dt = Local.from_utc_datetime(&dt.naive_utc());
    
    // Format based on how recent the timestamp is
    let now = Local::now();
    let duration = now.signed_duration_since(local_dt);
    
    if duration.num_days() == 0 {
        // Today - just show time
        local_dt.format("Today %H:%M").to_string()
    } else if duration.num_days() == 1 {
        // Yesterday
        local_dt.format("Yesterday %H:%M").to_string()
    } else if duration.num_days() < 7 {
        // This week - show day name
        local_dt.format("%a %H:%M").to_string()
    } else if duration.num_days() < 365 {
        // This year - show month and day
        local_dt.format("%b %d").to_string()
    } else {
        // Older - show full date
        local_dt.format("%m/%d/%y").to_string()
    }
}