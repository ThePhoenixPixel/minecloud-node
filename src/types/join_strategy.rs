use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum JoinStrategy {
    #[serde(rename = "fullest")]
    Fullest,

    #[serde(rename = "emptiest")]
    Emptiest,

    #[serde(rename = "round_robin")]
    RoundRobin,

    #[serde(rename = "random")]
    Random,
}

impl fmt::Display for JoinStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            JoinStrategy::Fullest => "fullest",
            JoinStrategy::Emptiest => "emptiest",
            JoinStrategy::RoundRobin => "round_robin",
            JoinStrategy::Random => "random",
        };
        write!(f, "{}", value)
    }
}
