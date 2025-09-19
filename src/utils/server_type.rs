use serde::{Deserialize, Serialize};

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
        match self {
            ServerType::Proxy => true,
            _ => false,
        }
    }
    pub fn is_backend_server(&self) -> bool {
        match self {
            ServerType::BackendServer => true,
            _ => false,
        }
    }
}
