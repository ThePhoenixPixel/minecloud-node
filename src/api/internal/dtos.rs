use bx::network::address::Address;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{EntityId, PlayerAction, PlayerRequest, Service};


#[derive(Deserialize)]
pub struct ServiceIdRequest {
    id: Uuid,
}

impl From<&ServiceIdRequest> for EntityId {
    fn from(value: &ServiceIdRequest) -> Self {
        value.id
    }
}

#[derive(Serialize, Debug)]
pub struct ServiceInfoResponse {
    name: String,
    address: Address,
    default_connect: bool,
    join_permission: String,
}

impl ServiceInfoResponse {

    pub fn new(service: &Service) -> ServiceInfoResponse {
        ServiceInfoResponse {
            name: service.get_name(),
            address: service.get_server_listener(),
            default_connect: service.get_task().default_connect(),
            join_permission: service.get_task().get_join_permission().to_string(),
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_address(&self) -> Address {
        self.address.clone()
    }

    pub fn is_default_connect(&self) -> bool {
        self.default_connect
    }

    pub fn get_join_permission(&self) -> String {
        self.join_permission.clone()
    }
}

impl From<&Service> for ServiceInfoResponse {
    fn from(service: &Service) -> Self {
        ServiceInfoResponse {
            name: service.get_name(),
            address: service.get_server_listener(),
            default_connect: service.get_task().default_connect(),
            join_permission: service.get_task().get_join_permission().to_string(),
        }
    }
}

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

