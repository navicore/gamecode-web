use leptos::*;
use crate::storage::{estimate_context_tokens, ContextState};
use crate::api::ChatMessage;

const MAX_CONTEXT_TOKENS: usize = 4096; // Adjust based on your models
const AUTO_COMPRESS_THRESHOLD: f32 = 0.8; // Compress at 80% full

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
            // TODO: Trigger compression
            web_sys::console::log_1(&"Auto-compression needed".into());
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
            </div>
        </div>
    }
}