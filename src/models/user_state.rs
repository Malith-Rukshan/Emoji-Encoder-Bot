use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BotState {
    Idle,
    AwaitingCustomEmoji { text: String },
}

pub type StateStorage = Arc<RwLock<HashMap<i64, BotState>>>;

pub fn create_state_storage() -> StateStorage {
    Arc::new(RwLock::new(HashMap::new()))
}

pub async fn get_user_state(storage: &StateStorage, user_id: i64) -> BotState {
    storage
        .read()
        .await
        .get(&user_id)
        .cloned()
        .unwrap_or(BotState::Idle)
}

pub async fn set_user_state(storage: &StateStorage, user_id: i64, state: BotState) {
    storage.write().await.insert(user_id, state);
}

pub async fn clear_user_state(storage: &StateStorage, user_id: i64) {
    storage.write().await.remove(&user_id);
}
