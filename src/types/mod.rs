use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::fmt;
use uuid::Uuid;

pub use installer::*;
pub use node::*;
pub use player::*;
pub use process::*;
pub use service::*;
pub use software::*;
pub use task::*;
pub use template::*;
pub use service_config::*;

mod group;
mod installer;
mod node;
mod task;
mod template;

mod player;
mod process;
mod service;
mod software;
mod service_config;

pub type EntityId = Uuid;

pub enum CloudUuid {
    Entity(EntityId),
    Str(String),
}

impl fmt::Display for CloudUuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CloudUuid::Entity(uuid) => write!(f, "{}", uuid),
            CloudUuid::Str(s) => write!(f, "{}", s),
        }
    }
}

impl From<EntityId> for CloudUuid {
    fn from(id: EntityId) -> Self {
        CloudUuid::Entity(id)
    }
}

impl From<&EntityId> for CloudUuid {
    fn from(id: &EntityId) -> Self {
        CloudUuid::Entity(id.clone())
    }
}

impl From<&str> for CloudUuid {
    fn from(s: &str) -> Self {
        CloudUuid::Str(s.to_string())
    }
}

impl From<String> for CloudUuid {
    fn from(s: String) -> Self {
        CloudUuid::Str(s)
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub enum ServerType {
    #[default]
    Proxy,
    BackendServer,
}

impl ServerType {
    pub fn to_string(&self) -> String {
        match self {
            ServerType::Proxy => String::from("Proxy"),
            ServerType::BackendServer => String::from("BackendServer"),
        }
    }
    pub fn is_proxy(&self) -> bool {
        *self == ServerType::Proxy
    }
    pub fn is_backend_server(&self) -> bool {
        *self == ServerType::BackendServer
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug, Default)]
pub enum ServiceStatus {
    #[default]
    Failed,

    Starting,
    Running,
    Stopping,
    Stopped,
}

impl ServiceStatus {
    pub fn to_string(&self) -> String {
        match self {
            ServiceStatus::Starting => String::from("Starting"),
            ServiceStatus::Running => String::from("Running"),
            ServiceStatus::Stopping => String::from("Stopping"),
            ServiceStatus::Stopped => String::from("Stopped"),
            ServiceStatus::Failed => String::from("Failed"),
        }
    }

    pub fn from_string(str: &str) -> ServiceStatus {
        match str {
            "Starting" => ServiceStatus::Starting,
            "Running" => ServiceStatus::Running,
            "Stopping" => ServiceStatus::Stopping,
            "Stopped" => ServiceStatus::Stopped,
            _ => ServiceStatus::Failed,
        }
    }

    pub fn is_starting(&self) -> bool {
        *self == ServiceStatus::Starting
    }

    pub fn is_running(&self) -> bool {
        *self == ServiceStatus::Running
    }

    pub fn is_stopping(&self) -> bool {
        *self == ServiceStatus::Stopping
    }

    pub fn is_stopped(&self) -> bool {
        *self == ServiceStatus::Stopped
    }

    pub fn is_failed(&self) -> bool {
        *self == ServiceStatus::Failed
    }
}
