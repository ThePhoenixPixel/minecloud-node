use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::fmt;
use std::path::Path;
use strum_macros::EnumIter;
use uuid::Uuid;

pub use group::*;
pub use installer::*;
pub use join_strategy::*;
pub use node::*;
pub use player::*;
pub use process::*;
pub use service::*;
pub use service_config::*;
pub use software_link::*;
pub use task::*;
pub use template::*;

mod group;
mod installer;
mod node;
mod task;
mod template;

mod join_strategy;
mod player;
mod process;
mod service;
mod service_config;
mod software_link;

/// EntityId for Service
pub type EntityId = Uuid;

/// SoftwareType
#[derive(Serialize, Deserialize, Clone, Debug, Default, EnumIter, Eq, Hash, PartialEq)]
pub enum SoftwareType {
    #[default]
    #[serde(rename = "proxy")]
    Proxy,

    #[serde(rename = "backend")]
    Backend,
}

impl fmt::Display for SoftwareType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            SoftwareType::Proxy => "proxy",
            SoftwareType::Backend => "backend",
        };
        write!(f, "{}", value)
    }
}

impl SoftwareType {
    pub fn is_proxy(&self) -> bool {
        *self == SoftwareType::Proxy
    }
    pub fn is_backend_server(&self) -> bool {
        *self == SoftwareType::Backend
    }
}

impl AsRef<Path> for SoftwareType {
    fn as_ref(&self) -> &Path {
        Path::new(match self {
            SoftwareType::Backend => "backend",
            SoftwareType::Proxy => "proxy",
        })
    }
}

/// ServiceStatus
#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
pub enum ServiceStatus {
    #[serde(rename = "failed")]
    Failed,

    #[serde(rename = "starting")]
    Starting,

    #[serde(rename = "running")]
    Running,

    #[serde(rename = "stopping")]
    Stopping,

    #[serde(rename = "stopped")]
    Stopped,
}

impl fmt::Display for ServiceStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            ServiceStatus::Failed => "failed",
            ServiceStatus::Starting => "starting",
            ServiceStatus::Running => "running",
            ServiceStatus::Stopping => "stopping",
            ServiceStatus::Stopped => "stopped",
        };
        write!(f, "{}", value)
    }
}
