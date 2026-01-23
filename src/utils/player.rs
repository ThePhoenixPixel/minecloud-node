use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct Player {
    uuid: Uuid,
    name: String,
}

impl Player {
    pub fn get_uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}
