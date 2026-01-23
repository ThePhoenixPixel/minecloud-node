use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub enum PlayerAction {
    #[default]
    Join,
    Leave,
}

impl PlayerAction {
    pub fn to_string(&self) -> String {
        match self {
            PlayerAction::Join => String::from("Join"),
            PlayerAction::Leave => String::from("Leave"),
        }
    }
}
