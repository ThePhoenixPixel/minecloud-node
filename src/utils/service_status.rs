use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Default)]
pub enum ServiceStatus {
    Start,
    Stop,
    Prepare,
    #[default]
    Null,
}

impl ServiceStatus {
    pub fn to_string(&self) -> String {
        match self {
            ServiceStatus::Start => String::from("Start"),
            ServiceStatus::Stop => String::from("Stop"),
            ServiceStatus::Prepare => String::from("Prepare"),
            ServiceStatus::Null => String::from("Null"),
        }
    }

    pub fn from_string(str: &str) -> ServiceStatus {
        match str {
            "Start" => ServiceStatus::Start,
            "Prepare" => ServiceStatus::Prepare,
            "Stop" => ServiceStatus::Stop,
            _ => ServiceStatus::Null,
        }
    }

    pub fn is_start(&self) -> bool {
        match self {
            ServiceStatus::Start => true,
            _ => false,
        }
    }

    pub fn is_stop(&self) -> bool {
        match self {
            ServiceStatus::Stop => true,
            _ => false,
        }
    }

    pub fn is_prepare(&self) -> bool {
        match self {
            ServiceStatus::Prepare => true,
            _ => false,
        }
    }

    pub fn is_null(&self) -> bool {
        match self {
            ServiceStatus::Null => true,
            _ => false,
        }
    }
}
