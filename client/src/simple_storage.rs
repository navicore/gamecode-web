use wasm_bindgen::prelude::*;
use web_sys::{window, Storage};
use serde::{Serialize, Deserialize};
use crate::storage::{StoredConversation, ContextState, ConversationMetadata, ConversationRef};

#[derive(Clone)]
pub struct SimpleStorage;

impl SimpleStorage {
    pub fn new() -> Self {
        web_sys::console::log_1(&"Using localStorage for conversation storage".into());
        Self
    }
    
    fn get_storage() -> Option<Storage> {
        window()?.local_storage().ok()?
    }
    
    pub fn save_conversation(&self, conversation: &StoredConversation) -> Result<(), JsValue> {
        let storage = Self::get_storage().ok_or("No localStorage")?;
        let key = format!("conversation_{}", conversation.id);
        let value = serde_json::to_string(conversation)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        web_sys::console::log_1(&format!("Saving to localStorage key: {}", key).into());
        storage.set_item(&key, &value)?;
        web_sys::console::log_1(&"Saved to localStorage".into());
        Ok(())
    }
    
    pub fn load_conversation(&self, id: &str) -> Result<Option<StoredConversation>, JsValue> {
        let storage = Self::get_storage().ok_or("No localStorage")?;
        let key = format!("conversation_{}", id);
        
        web_sys::console::log_1(&format!("Loading from localStorage key: {}", key).into());
        
        match storage.get_item(&key)? {
            Some(value) => {
                let conversation = serde_json::from_str(&value)
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
                web_sys::console::log_1(&"Loaded from localStorage".into());
                Ok(Some(conversation))
            }
            None => {
                web_sys::console::log_1(&"No conversation found in localStorage".into());
                Ok(None)
            }
        }
    }
    
    pub fn list_conversations(&self, limit: u32) -> Result<Vec<ConversationRef>, JsValue> {
        let storage = Self::get_storage().ok_or("No localStorage")?;
        let mut conversations = Vec::new();
        
        // Iterate through all keys in localStorage
        let length = storage.length()?;
        for i in 0..length {
            if let Some(key) = storage.key(i)? {
                if key.starts_with("conversation_") {
                    if let Some(value) = storage.get_item(&key)? {
                        if let Ok(conv) = serde_json::from_str::<StoredConversation>(&value) {
                            // Extract first user message for preview
                            let preview = conv.notebook.cells.iter()
                                .find_map(|cell| match &cell.content {
                                    crate::notebook::CellContent::UserInput { text } => Some(text.clone()),
                                    _ => None
                                })
                                .unwrap_or_else(|| "Empty conversation".to_string());
                            
                            conversations.push(ConversationRef {
                                id: conv.id,
                                title: conv.metadata.title,
                                modified_at: conv.metadata.modified_at,
                                preview: preview.chars().take(50).collect::<String>() + if preview.len() > 50 { "..." } else { "" },
                            });
                        }
                    }
                }
            }
        }
        
        // Sort by modified date (newest first)
        conversations.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));
        
        // Apply limit
        conversations.truncate(limit as usize);
        
        Ok(conversations)
    }
    
    pub fn delete_conversation(&self, id: &str) -> Result<(), JsValue> {
        let storage = Self::get_storage().ok_or("No localStorage")?;
        let key = format!("conversation_{}", id);
        storage.remove_item(&key)?;
        Ok(())
    }
    
    pub fn clear_all_conversations(&self) -> Result<(), JsValue> {
        let storage = Self::get_storage().ok_or("No localStorage")?;
        let mut keys_to_remove = Vec::new();
        
        // Collect all conversation keys
        let length = storage.length()?;
        for i in 0..length {
            if let Some(key) = storage.key(i)? {
                if key.starts_with("conversation_") {
                    keys_to_remove.push(key);
                }
            }
        }
        
        // Remove all conversation keys
        for key in keys_to_remove {
            storage.remove_item(&key)?;
        }
        
        Ok(())
    }
}