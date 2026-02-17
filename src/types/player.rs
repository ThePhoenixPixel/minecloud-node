use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct Player {
    id: u64,
    name: String,
    uuid: Uuid,
    session: Option<PlayerSession>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct PlayerSession {
    id: u64,
    service_uuid: Uuid,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct PlayerRequest {
    uuid: Uuid,
    name: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub enum PlayerAction {
    #[default]
    Join,
    Leave,
}

impl Player {
    pub fn new(id: u64, name: String, uuid: Uuid, session: Option<PlayerSession>) -> Player {
        Player {
            id,
            name,
            uuid,
            session,
        }
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn get_uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn get_uuid_str(&self) -> String {
        self.uuid.to_string()
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_session(&self) -> Option<PlayerSession> {
        self.session.clone()
    }

    pub fn set_session(&mut self, session: PlayerSession) {
        self.session = Some(session);
    }

    pub fn clear_session(&mut self) {
        self.session = None;
    }
}

impl PlayerSession {
    pub fn new(id: u64, service_uuid: Uuid) -> PlayerSession {
        PlayerSession {
            id,
            service_uuid,
        }
    }
    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn set_id(&mut self, id: u64) {
        self.id = id
    }

    pub fn get_service_uuid(&self) -> Uuid {
        self.service_uuid.clone()
    }

    pub fn set_service_id(&mut self, uuid: &Uuid) {
        self.service_uuid = uuid.clone()
    }
}

impl PlayerRequest {
    pub fn get_uuid(&self) -> Uuid {
        self.uuid
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}

impl From<PlayerRequest> for Player {
    fn from(value: PlayerRequest) -> Self {
        Player::new(0, value.get_name(), value.get_uuid(), None)
    }
}

impl PlayerAction {
    pub fn to_string(&self) -> String {
        match self {
            PlayerAction::Join => String::from("Join"),
            PlayerAction::Leave => String::from("Leave"),
        }
    }
}
