use std::collections::HashMap;
use std::sync::Mutex;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use tokio::sync::broadcast;
use serde_json;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub last_seen: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub username: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

pub struct ChatState {
    users: Mutex<HashMap<String, User>>,
    tx: broadcast::Sender<ChatMessage>,
    websockets: Mutex<Vec<UnboundedSender<String>>>,
}

impl ChatState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            users: Mutex::new(HashMap::new()),
            tx,
            websockets: Mutex::new(Vec::new()),
        }
    }

    pub fn add_user(&self, username: String) {
        let mut users = self.users.lock().unwrap();
        users.insert(
            username.clone(),
            User {
                username: username.clone(),
                last_seen: Utc::now(),
            },
        );
    }

    pub fn remove_user(&self, username: &str) {
        let mut users = self.users.lock().unwrap();
        users.remove(username);
    }

    pub fn get_users(&self) -> Vec<User> {
        let users = self.users.lock().unwrap();
        users.values().cloned().collect()
    }

    pub fn broadcast_message(&self, message: ChatMessage) {
        let _ = self.tx.send(message);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ChatMessage> {
        self.tx.subscribe()
    }

    pub fn register_ws(&self, sender: UnboundedSender<String>) -> usize {
        let mut websockets = self.websockets.lock().unwrap();
        websockets.push(sender);
        websockets.len() - 1
    }

    pub fn unregister_ws(&self, id: usize) {
        let mut websockets = self.websockets.lock().unwrap();
        if id < websockets.len() {
            websockets.remove(id);
        }
    }

    pub fn broadcast_user_list(&self) {
        let users = self.get_users();
        let msg = serde_json::json!({
            "type": "userList",
            "users": users
        });
        let msg_str = serde_json::to_string(&msg).unwrap();
        let mut websockets = self.websockets.lock().unwrap();
        websockets.retain(|tx| tx.send(msg_str.clone()).is_ok());
    }
}