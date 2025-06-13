use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{IdbDatabase, IdbRequest, IdbTransactionMode, IdbOpenDbRequest};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use crate::notebook::Notebook;
use std::rc::Rc;
use js_sys;

const DB_NAME: &str = "gamecode_conversations";
const DB_VERSION: u32 = 1;
const STORE_NAME: &str = "conversations";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredConversation {
    pub id: String,
    pub notebook: Notebook,
    pub context_state: ContextState,
    pub metadata: ConversationMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextState {
    pub compressed_summaries: Vec<String>,
    pub active_messages: Vec<crate::api::ChatMessage>,
    pub total_tokens: usize,
    pub compression_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMetadata {
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub title: String,
    pub model: String,
    pub provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationRef {
    pub id: String,
    pub title: String,
    pub modified_at: DateTime<Utc>,
    pub preview: String,
}

#[derive(Clone)]
pub struct ConversationStorage {
    db: Rc<IdbDatabase>,
}

impl ConversationStorage {
    pub async fn new() -> Result<Self, JsValue> {
        web_sys::console::log_1(&"ConversationStorage::new() called".into());
        
        let window = match web_sys::window() {
            Some(w) => w,
            None => {
                web_sys::console::error_1(&"No window object".into());
                return Err("No window".into());
            }
        };
        
        let idb = match window.indexed_db() {
            Ok(Some(idb)) => idb,
            Ok(None) => {
                web_sys::console::error_1(&"IndexedDB not available".into());
                return Err("No IndexedDB".into());
            }
            Err(e) => {
                web_sys::console::error_1(&format!("Error accessing IndexedDB: {:?}", e).into());
                return Err(e);
            }
        };
        
        // Open database
        web_sys::console::log_1(&format!("Opening database: {} version: {}", DB_NAME, DB_VERSION).into());
        let open_request = idb.open_with_u32(DB_NAME, DB_VERSION)?;
        
        // Set up upgrade handler before converting to promise
        let onupgradeneeded = Closure::wrap(Box::new(move |event: web_sys::Event| {
            web_sys::console::log_1(&"⚠️ IndexedDB upgrade needed event fired".into());
            let target = event.target().expect("Event should have target");
            let request: &IdbRequest = target.dyn_ref().expect("Target should be IdbRequest");
            let result = request.result().expect("Request should have result");
            let db: &IdbDatabase = result.dyn_ref().expect("Result should be IdbDatabase");
            
            // Create object store if it doesn't exist
            match db.create_object_store(STORE_NAME) {
                Ok(_) => {
                    web_sys::console::log_1(&format!("✅ Created object store: {}", STORE_NAME).into());
                },
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to create store: {:?}", e).into());
                }
            }
        }) as Box<dyn FnMut(_)>);
        
        open_request.set_onupgradeneeded(Some(onupgradeneeded.as_ref().unchecked_ref()));
        onupgradeneeded.forget();
        
        // Wait for the request to complete using JsFuture
        web_sys::console::log_1(&"Waiting for database to open...".into());
        let promise = open_request.unchecked_ref::<js_sys::Promise>();
        let db = JsFuture::from(promise.clone()).await?;
        let db: IdbDatabase = db.dyn_into()?;
        
        web_sys::console::log_1(&format!("✅ IndexedDB opened successfully: {}", DB_NAME).into());
        
        Ok(Self {
            db: Rc::new(db),
        })
    }
    
    pub async fn save_conversation(&self, conversation: &StoredConversation) -> Result<(), JsValue> {
        web_sys::console::log_1(&format!("Saving conversation to IndexedDB: {}", conversation.id).into());
        
        let transaction = self.db.transaction_with_str_and_mode(
            STORE_NAME,
            IdbTransactionMode::Readwrite,
        )?;
        let store = transaction.object_store(STORE_NAME)?;
        
        // Serialize conversation
        let value = serde_wasm_bindgen::to_value(conversation)?;
        
        // Use ID as key
        let key = JsValue::from_str(&conversation.id);
        
        web_sys::console::log_1(&format!("Storing with key: {}", conversation.id).into());
        
        let request = store.put_with_key(&value, &key)?;
        let promise = request.unchecked_ref::<js_sys::Promise>();
        let _ = JsFuture::from(promise.clone()).await?;
        
        web_sys::console::log_1(&"IndexedDB save completed".into());
        
        Ok(())
    }
    
    pub async fn load_conversation(&self, id: &str) -> Result<Option<StoredConversation>, JsValue> {
        let transaction = self.db.transaction_with_str(STORE_NAME)?;
        let store = transaction.object_store(STORE_NAME)?;
        
        let key = JsValue::from_str(id);
        let request = store.get(&key)?;
        let promise = request.unchecked_ref::<js_sys::Promise>();
        let result = JsFuture::from(promise.clone()).await?;
        
        if result.is_undefined() || result.is_null() {
            Ok(None)
        } else {
            let conversation: StoredConversation = serde_wasm_bindgen::from_value(result)?;
            Ok(Some(conversation))
        }
    }
    
    pub async fn list_conversations(&self, _limit: u32) -> Result<Vec<ConversationRef>, JsValue> {
        // TODO: Implement proper cursor iteration
        // For now, return empty list
        Ok(Vec::new())
    }
    
    pub async fn delete_conversation(&self, id: &str) -> Result<(), JsValue> {
        let transaction = self.db.transaction_with_str_and_mode(
            STORE_NAME,
            IdbTransactionMode::Readwrite,
        )?;
        let store = transaction.object_store(STORE_NAME)?;
        
        let key = JsValue::from_str(id);
        let request = store.delete(&key)?;
        let promise = request.unchecked_ref::<js_sys::Promise>();
        let _ = JsFuture::from(promise.clone()).await?;
        
        Ok(())
    }
    
    pub async fn clear_all_conversations(&self) -> Result<(), JsValue> {
        let transaction = self.db.transaction_with_str_and_mode(
            STORE_NAME,
            IdbTransactionMode::Readwrite,
        )?;
        let store = transaction.object_store(STORE_NAME)?;
        
        let request = store.clear()?;
        let promise = request.unchecked_ref::<js_sys::Promise>();
        let _ = JsFuture::from(promise.clone()).await?;
        
        Ok(())
    }
}

// Token estimation (rough approximation)
pub fn estimate_tokens(text: &str) -> usize {
    // Rough estimate: ~4 characters per token for English
    // This is very approximate and should be refined based on the model
    text.len() / 4
}

pub fn estimate_message_tokens(message: &crate::api::ChatMessage) -> usize {
    estimate_tokens(&message.role) + estimate_tokens(&message.content) + 3 // overhead
}

pub fn estimate_context_tokens(messages: &[crate::api::ChatMessage]) -> usize {
    messages.iter().map(estimate_message_tokens).sum()
}