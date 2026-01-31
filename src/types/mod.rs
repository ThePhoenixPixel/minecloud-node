use std::cmp::PartialEq;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod group;
pub mod task;
pub mod template;
//pub mod node;
pub mod installer;

pub mod service;
//pub mod services_all;
//pub mod services_local;
pub mod software;


pub type EntityId = Uuid;


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
        self == ServerType::Proxy
    }
    pub fn is_backend_server(&self) -> bool {
        self == ServerType::BackendServer
    }
}


impl PartialEq<ServerType> for &ServerType {
    fn eq(&self, other: &ServerType) -> bool {
        self == other
    }
}




#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Default)]
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
            ServiceStatus::Starting     => String::from("Starting"),
            ServiceStatus::Running      => String::from("Running"),
            ServiceStatus::Stopping     => String::from("Stopping"),
            ServiceStatus::Stopped      => String::from("Stopped"),
            ServiceStatus::Failed       => String::from("Failed"),
        }
    }

    pub fn from_string(str: &str) -> ServiceStatus {
        match str {
            "Starting"      => ServiceStatus::Starting,
            "Running"       => ServiceStatus::Running,
            "Stopping"      => ServiceStatus::Stopping,
            "Stopped"       => ServiceStatus::Stopped,
            _               => ServiceStatus::Failed,
        }
    }

    pub fn is_starting(&self) -> bool {
        self == ServiceStatus::Starting
    }

    pub fn is_running(&self) -> bool {
        self == ServiceStatus::Running
    }

    pub fn is_stopping(&self) -> bool {
        self == ServiceStatus::Stopping
    }

    pub fn is_stopped(&self) -> bool {
        self == ServiceStatus::Stopped
    }

    pub fn is_failed(&self) -> bool {
        self == ServiceStatus::Failed
    }
}

impl PartialEq<ServiceStatus> for &ServiceStatus {
    fn eq(&self, other: &ServiceStatus) -> bool {
        self == other
    }
}


