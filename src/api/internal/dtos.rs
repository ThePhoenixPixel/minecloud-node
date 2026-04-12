use bx::network::address::Address;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::types::{EntityId, PlayerAction, PlayerRequest, Service};

#[derive(Debug, Deserialize)]
pub struct IncomingMessage {
    /// z.B. "get_backend_services" | "service_online" | "service_shutdown" | "player_action"
    #[serde(rename = "type")]
    msg_type: MessageType,

    service_id: Uuid,

    #[serde(default)]
    data: Value,
}

impl IncomingMessage {
    pub fn get_msg_typ(&self) -> &MessageType {
        &self.msg_type
    }

    pub fn get_service_id(&self) -> Uuid {
        self.service_id
    }

    pub fn get_data(&self) -> &Value {
        &self.data
    }
}

#[derive(Debug, Serialize)]
pub struct OutgoingMessage {
    #[serde(rename = "type")]
    msg_type: MessageType,

    success: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl OutgoingMessage {
    pub fn ok(msg_type: impl Into<MessageType>, data: Option<Value>) -> String
    {
        serde_json::to_string(&Self {
            msg_type: msg_type.into(),
            success: true,
            data,
            error: None,
        })
            .unwrap()
    }

    pub fn err(msg_type: impl Into<MessageType>, e: impl ToString) -> String {
        serde_json::to_string(&Self {
            msg_type: msg_type.into(),
            success: false,
            data: None,
            error: Some(e.to_string()),
        })
            .unwrap()
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum MessageType {
    auth,
    get_online_backend_services,
    service_online,
    service_shutdown,
    shutdown,
    player_action,
    error,

    add_server,
    remove_server,

}

impl PartialEq<MessageType> for &MessageType {
    fn eq(&self, other: &MessageType) -> bool {
        **self == *other
    }
}


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
            name: service.get_name().to_string(),
            address: service.get_server_listener().clone(),
            default_connect: service.default_connect(),
            join_permission: service.get_join_permission().to_string(),
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
    fn from(service: &Service) -> ServiceInfoResponse {
        ServiceInfoResponse::new(service)
    }
}

impl From<Service> for ServiceInfoResponse {
    fn from(service: Service) -> ServiceInfoResponse {
        ServiceInfoResponse::new(&service)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerActionRequest {
    action: PlayerAction,
    service_uuid: Uuid,
    player_uuid: Uuid,
    player_name: String,
}

impl PlayerActionRequest {
    pub fn get_action(&self) -> &PlayerAction {
        &self.action
    }

    pub fn get_player_name(&self) -> &str {
        &self.player_name
    }

    pub fn get_player_uuid(&self) -> Uuid {
        self.player_uuid
    }
    
    pub fn get_service_uuid(&self) -> Uuid {
        self.service_uuid
    }
}
