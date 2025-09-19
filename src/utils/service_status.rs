use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum ServiceStatus {
    Start,
    Stop,
    Prepare,
}

impl ServiceStatus {
    pub fn to_string(&self) -> String {
        match self {
            ServiceStatus::Start => String::from("Start"),
            ServiceStatus::Stop => String::from("Stop"),
            ServiceStatus::Prepare => String::from("Prepare"),
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
}
