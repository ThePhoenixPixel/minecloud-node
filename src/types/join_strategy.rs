use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum JoinStrategy {
    Fullest,
    Emptiest,
    RoundRobin,
    Random,
}

impl JoinStrategy {
    pub fn from_string(s: &str) -> Self {
        match s {
            "Fullest" => JoinStrategy::Fullest,
            "Emptiest" => JoinStrategy::Emptiest,
            "RoundRobin" => JoinStrategy::RoundRobin,
            "Random" => JoinStrategy::Random,
            _ => JoinStrategy::Random,
        }
    }

    pub fn too_string(value: &JoinStrategy) -> &str {
        match value {
            JoinStrategy::Fullest => "Fullest",
            JoinStrategy::Emptiest => "Emptiest",
            JoinStrategy::RoundRobin => "RoundRobin",
            JoinStrategy::Random => "Random",
        }
    }
}

impl From<&str> for JoinStrategy {
    fn from(s: &str) -> Self {
        JoinStrategy::from_string(s)
    }
}

impl From<String> for JoinStrategy {
    fn from(s: String) -> Self {
        JoinStrategy::from_string(s.as_str())
    }
}

impl From<JoinStrategy> for String {
    fn from(value: JoinStrategy) -> Self {
        JoinStrategy::too_string(&value).to_string()
    }
}

impl From<&JoinStrategy> for String {
    fn from(value: &JoinStrategy) -> Self {
        JoinStrategy::too_string(value).to_string()
    }
}

