use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Installer {
    #[serde(rename = "install_all")]
    InstallAll,

    #[serde(rename = "install_all_desc")]
    InstallAllDesc,

    #[serde(rename = "install_random")]
    InstallRandom,

    #[serde(rename = "install_random_with_priority")]
    InstallRandomWithPriority,
}

impl fmt::Display for Installer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Installer::InstallAll => "install_all",
            Installer::InstallAllDesc => "install_all_desc",
            Installer::InstallRandom => "install_random",
            Installer::InstallRandomWithPriority => "install_random_with_priority",
        };
        write!(f, "{}", value)
    }
}
