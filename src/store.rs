use crate::models::Item;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    items: Arc<RwLock<HashMap<String, Item>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            items: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&self, name: String) -> Item {
        let id = Uuid::new_v4().to_string(); // server-vergebene, nicht erratbare ID
        let item = Item {
            id: id.clone(),
            name,
        };
        self.items.write().unwrap().insert(id, item.clone()); // lock → insert → guard fällt
        item
    }

    pub fn get(&self, id: &str) -> Option<Item> {
        self.items.read().unwrap().get(id).cloned()
    }

    pub fn list(&self) -> Vec<Item> {
        self.items.read().unwrap().values().cloned().collect()
    }

    pub fn delete(&self, id: &str) -> bool {
        self.items.write().unwrap().remove(id).is_some() // true wenn was entfernt wurde
    }

    pub fn update(&self, id: &str, name: String) -> Option<Item> {
        let mut items = self.items.write().unwrap();
        let item = items.get_mut(id)?; // None → früh raus → Handler gibt 404
        item.name = name;
        Some(item.clone())
    }
}
