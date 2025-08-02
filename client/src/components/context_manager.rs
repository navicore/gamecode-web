use leptos::*;
use crate::storage::{estimate_context_tokens, ContextState};
use crate::api::ChatMessage;

const MAX_CONTEXT_TOKENS: usize = 4096; // Adjust based on your models
const AUTO_COMPRESS_THRESHOLD: f32 = 0.85; // Compress at 85% full (leaves room for response)

#[derive(Clone)]
pub struct ContextManager {
    messages: RwSignal<Vec<ChatMessage>>,
    compressed_summaries: RwSignal<Vec<String>>,
    total_tokens: RwSignal<usize>,
    compression_count: RwSignal<u32>,
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            messages: create_rw_signal(Vec::new()),
            compressed_summaries: create_rw_signal(Vec::new()),
            total_tokens: create_rw_signal(0),
            compression_count: create_rw_signal(0),
        }
    }
    
    pub fn restore_state(&self, state: ContextState) {
        self.messages.set(state.active_messages);
        self.compressed_summaries.set(state.compressed_summaries);
        self.total_tokens.set(state.total_tokens);
        self.compression_count.set(state.compression_count);
    }
    
    pub fn from_state(state: ContextState) -> Self {
        Self {
            messages: create_rw_signal(state.active_messages),
            compressed_summaries: create_rw_signal(state.compressed_summaries),
            total_tokens: create_rw_signal(state.total_tokens),
            compression_count: create_rw_signal(state.compression_count),
        }
    }
    
    pub fn to_state(&self) -> ContextState {
        ContextState {
            active_messages: self.messages.get(),
            compressed_summaries: self.compressed_summaries.get(),
            total_tokens: self.total_tokens.get(),
            compression_count: self.compression_count.get(),
        }
    }
    
    pub fn add_message(&self, message: ChatMessage) {
        self.messages.update(|msgs| msgs.push(message.clone()));
        self.update_token_count();
        
        // Check if we need auto-compression
        if self.should_auto_compress() {
            web_sys::console::log_1(&"Auto-compression triggered".into());
            if self.compress_context() {
                web_sys::console::log_1(&"Auto-compression successful".into());
            } else {
                web_sys::console::log_1(&"Auto-compression failed or skipped".into());
            }
        }
    }
    
    pub fn get_context_for_request(&self) -> Vec<ChatMessage> {
        let mut context = Vec::new();
        
        // Add compressed summaries as system messages
        for summary in self.compressed_summaries.get() {
            context.push(ChatMessage {
                role: "system".to_string(),
                content: format!("Previous conversation summary: {}", summary),
            });
        }
        
        // Add active messages
        context.extend(self.messages.get());
        
        context
    }
    
    fn update_token_count(&self) {
        let messages_tokens = estimate_context_tokens(&self.messages.get());
        let summary_tokens: usize = self.compressed_summaries.get()
            .iter()
            .map(|s| crate::storage::estimate_tokens(s))
            .sum();
        
        self.total_tokens.set(messages_tokens + summary_tokens);
    }
    
    fn should_auto_compress(&self) -> bool {
        let usage = self.total_tokens.get() as f32 / MAX_CONTEXT_TOKENS as f32;
        usage > AUTO_COMPRESS_THRESHOLD
    }
    
    pub fn get_usage_percentage(&self) -> f32 {
        (self.total_tokens.get() as f32 / MAX_CONTEXT_TOKENS as f32) * 100.0
    }
    
    pub fn get_total_tokens(&self) -> usize {
        self.total_tokens.get()
    }
    
    pub fn compress_context(&self) -> bool {
        let current_messages = self.messages.get();
        
        // Don't compress if we have too few messages
        if current_messages.len() < 6 {
            web_sys::console::log_1(&"Not enough messages to compress (need at least 6)".into());
            return false;
        }
        
        // Calculate split point - keep last 30% of messages or at least 4 messages
        let total_msgs = current_messages.len();
        let keep_count = std::cmp::max(4, (total_msgs as f32 * 0.3).ceil() as usize);
        let compress_count = total_msgs - keep_count;
        
        if compress_count < 4 {
            web_sys::console::log_1(&format!(
                "Not enough messages to compress effectively (would compress {} messages)", 
                compress_count
            ).into());
            return false;
        }
        
        // Split messages into compress and keep
        let messages_to_compress: Vec<ChatMessage> = current_messages
            .iter()
            .take(compress_count)
            .cloned()
            .collect();
        
        let messages_to_keep: Vec<ChatMessage> = current_messages
            .iter()
            .skip(compress_count)
            .cloned()
            .collect();
        
        // Create an intelligent summary
        let summary = self.create_summary(&messages_to_compress);
        
        // Calculate token savings
        let original_tokens = estimate_context_tokens(&current_messages);
        let summary_tokens = crate::storage::estimate_tokens(&summary);
        let kept_tokens = estimate_context_tokens(&messages_to_keep);
        let new_total = summary_tokens + kept_tokens;
        
        web_sys::console::log_1(&format!(
            "Compression: {} messages → summary + {} messages. Tokens: {} → {} (saved {})",
            total_msgs,
            keep_count,
            original_tokens,
            new_total,
            original_tokens - new_total
        ).into());
        
        // Only compress if we actually save tokens
        if new_total >= (original_tokens as f32 * 0.9) as usize {
            web_sys::console::log_1(&"Compression wouldn't save enough tokens, skipping".into());
            return false;
        }
        
        // Update state
        self.compressed_summaries.update(|sums| sums.push(summary));
        self.messages.set(messages_to_keep);
        self.compression_count.update(|c| *c += 1);
        self.update_token_count();
        
        true
    }
    
    fn create_summary(&self, messages: &[ChatMessage]) -> String {
        let mut summary_parts = Vec::new();
        
        // Group messages into conversation turns
        let mut conversation_topics = Vec::new();
        let mut current_topic = Vec::new();
        
        for msg in messages {
            current_topic.push(msg);
            
            // Start new topic after assistant response
            if msg.role == "assistant" && !current_topic.is_empty() {
                conversation_topics.push(current_topic.clone());
                current_topic.clear();
            }
        }
        
        // Add any remaining messages
        if !current_topic.is_empty() {
            conversation_topics.push(current_topic);
        }
        
        // Summarize each topic
        for topic_msgs in conversation_topics.iter().take(5) { // Limit to 5 most important topics
            let mut topic_summary = String::new();
            
            for msg in topic_msgs {
                match msg.role.as_str() {
                    "user" => {
                        // Extract key intent from user message
                        let content = &msg.content;
                        let summary = if content.len() > 150 {
                            // Try to find a natural break point
                            if let Some(pos) = content[..150].rfind(". ") {
                                format!("{}.", &content[..pos])
                            } else if let Some(pos) = content[..150].rfind("? ") {
                                format!("{}?", &content[..pos])
                            } else {
                                format!("{}...", &content[..147])
                            }
                        } else {
                            content.clone()
                        };
                        
                        if !topic_summary.is_empty() {
                            topic_summary.push_str(" → ");
                        }
                        topic_summary.push_str(&format!("User: {}", summary));
                    }
                    "assistant" => {
                        // Extract key response from assistant
                        let content = &msg.content;
                        let summary = if content.len() > 200 {
                            // Look for code blocks or important markers
                            if content.contains("```") {
                                "provided code implementation"
                            } else if content.contains("error") || content.contains("Error") {
                                "addressed an error"
                            } else if content.contains("fixed") || content.contains("Fixed") {
                                "fixed an issue"
                            } else if content.contains("implemented") || content.contains("added") {
                                "implemented requested features"
                            } else {
                                // Generic summary
                                "provided detailed response"
                            }
                        } else if content.len() > 100 {
                            "gave explanation"
                        } else {
                            "responded"
                        };
                        
                        topic_summary.push_str(&format!(" → Assistant: {}", summary));
                    }
                    "system" => {
                        // Include system messages if important
                        if msg.content.len() < 100 {
                            topic_summary.push_str(&format!(" [System: {}]", msg.content));
                        }
                    }
                    _ => {}
                }
            }
            
            if !topic_summary.is_empty() {
                summary_parts.push(topic_summary);
            }
        }
        
        // Create final summary
        let mut final_summary = String::from("Previous conversation context (compressed): ");
        
        if summary_parts.is_empty() {
            final_summary.push_str("General discussion about the project.");
        } else {
            final_summary.push_str(&summary_parts.join(" | "));
        }
        
        // Add note about compression
        final_summary.push_str(&format!(
            " [Compressed {} messages to save context space]",
            messages.len()
        ));
        
        final_summary
    }
    
    pub fn clear_context(&self) {
        self.messages.set(Vec::new());
        self.compressed_summaries.set(Vec::new());
        self.total_tokens.set(0);
        self.compression_count.set(0);
    }
}

#[component]
pub fn ContextDisplay(
    context_manager: ContextManager,
    on_compress: impl Fn() + 'static,
    on_clear: impl Fn() + 'static,
    #[prop(optional)] on_logout: Option<impl Fn() + 'static>,
) -> impl IntoView {
    let cm1 = context_manager.clone();
    let cm2 = context_manager.clone();
    let cm3 = context_manager.clone();
    let cm4 = context_manager.clone();
    let cm5 = context_manager.clone();
    let cm6 = context_manager.clone();
    
    view! {
        <div class="context-footer">
            <div class="context-progress-container">
                <div class="context-progress-bar">
                    <div 
                        class="context-progress-fill"
                        style:width=move || format!("{}%", cm1.get_usage_percentage())
                        class=("warning", move || cm2.get_usage_percentage() > 70.0)
                        class=("critical", move || cm3.get_usage_percentage() > 90.0)
                    >
                        <span class="context-progress-glow"></span>
                    </div>
                </div>
                <span class="context-token-text">
                    {move || {
                        let tokens = cm4.total_tokens.get();
                        let percentage = cm5.get_usage_percentage();
                        if tokens < 1000 {
                            format!("{} tokens", tokens)
                        } else {
                            format!("{:.1}k / {:.0}k ({:.0}%)", 
                                tokens as f64 / 1000.0,
                                MAX_CONTEXT_TOKENS as f64 / 1000.0,
                                percentage
                            )
                        }
                    }}
                </span>
            </div>
            
            <div class="context-footer-actions">
                {move || if cm6.compression_count.get() > 0 {
                    view! {
                        <span class="compression-indicator" title={format!("Compressed {} times", cm6.compression_count.get())}>
                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                <path d="M4 14l1 1 5-5-5-5-1 1 4 4-4 4z"/>
                                <path d="M10 14l1 1 5-5-5-5-1 1 4 4-4 4z"/>
                            </svg>
                            {cm6.compression_count.get()}
                        </span>
                    }.into_view()
                } else {
                    view! { <span></span> }.into_view()
                }}
                
                <button 
                    class="context-action-btn compress"
                    on:click=move |_| on_compress()
                    title="Compress older messages to save space"
                >
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M22 12h-6l4-4M2 12h6l-4 4M12 2v6l4-4M12 22v-6l-4 4"/>
                    </svg>
                </button>
                <button 
                    class="context-action-btn clear"
                    on:click=move |_| on_clear()
                    title="Clear all context and start fresh"
                >
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <polyline points="3 6 5 6 21 6"/>
                        <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
                    </svg>
                </button>
                
                {if let Some(logout_fn) = on_logout {
                    view! {
                        <>
                            <div class="context-actions-separator"></div>
                            <button 
                                class="context-action-btn logout"
                                on:click=move |_| logout_fn()
                                title="Sign out"
                            >
                                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                    <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/>
                                    <polyline points="16 17 21 12 16 7"/>
                                    <line x1="21" y1="12" x2="9" y2="12"/>
                                </svg>
                            </button>
                        </>
                    }.into_view()
                } else {
                    view! { <span></span> }.into_view()
                }}
            </div>
        </div>
    }
}