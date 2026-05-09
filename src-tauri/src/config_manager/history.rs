use super::model::{ConfigHistory, ConfigSnapshot, HistoryAction};
use chrono::Utc;
use std::collections::VecDeque;
use std::sync::Mutex;

const MAX_HISTORY_SIZE: usize = 50;

lazy_static::lazy_static! {
    static ref HISTORY_STACK: Mutex<VecDeque<ConfigHistory>> = Mutex::new(VecDeque::new());
}

pub fn add_history(snapshot: ConfigSnapshot, action: HistoryAction, operator: Option<String>) {
    let mut history = HISTORY_STACK.lock().unwrap();
    let history_item = ConfigHistory {
        id: uuid::Uuid::new_v4().to_string(),
        snapshot,
        action,
        operator,
    };
    
    history.push_front(history_item);
    
    if history.len() > MAX_HISTORY_SIZE {
        history.pop_back();
    }
}

pub fn get_history(page: usize, page_size: usize) -> Vec<ConfigHistory> {
    let history = HISTORY_STACK.lock().unwrap();
    let start = page * page_size;
    
    history
        .iter()
        .skip(start)
        .take(page_size)
        .cloned()
        .collect()
}

pub fn get_history_by_id(id: &str) -> Option<ConfigHistory> {
    let history = HISTORY_STACK.lock().unwrap();
    history.iter().find(|h| h.id == id).cloned()
}

pub fn get_history_count() -> usize {
    HISTORY_STACK.lock().unwrap().len()
}

pub fn clear_history() {
    HISTORY_STACK.lock().unwrap().clear();
}

pub fn create_snapshot(config: &super::model::AppConfig, description: Option<String>) -> ConfigSnapshot {
    let items = config
        .items
        .iter()
        .map(|(key, value)| super::model::ConfigItem {
            key: key.clone(),
            value: value.clone(),
            category: config
                .categories
                .iter()
                .find(|c| c.items.contains(key))
                .map(|c| c.name.clone())
                .unwrap_or_else(|| "other".to_string()),
            updated_at: config.last_updated,
        })
        .collect();
    
    ConfigSnapshot {
        version: config.version.clone(),
        items,
        created_at: Utc::now(),
        description,
    }
}