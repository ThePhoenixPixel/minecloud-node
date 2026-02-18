use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{PlayerAction, PlayerRequest};



#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerActionRequest {
    action: PlayerAction,
    service_uuid: Uuid,
    player: PlayerRequest,
}

impl PlayerActionRequest {
    pub fn get_action(&self) -> PlayerAction {
        self.action.clone()
    }
    pub fn get_player_req(&self) -> PlayerRequest {
        self.player.clone()
    }
    pub fn get_service_uuid(&self) -> Uuid {
        self.service_uuid.clone()
    }
}


