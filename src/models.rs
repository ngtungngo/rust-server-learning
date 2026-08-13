use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone)]
pub struct Item {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize)]
pub struct CreateItem {
    pub name: String,
}
