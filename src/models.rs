use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Item {
    pub id: String,
}

#[derive(Deserialize)]
pub struct CreateItem {
    pub name: String,
}
