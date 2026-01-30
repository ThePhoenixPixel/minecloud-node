use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Installer {
    InstallAll,
    InstallAllDesc,
    InstallRandom,
    InstallRandomWithPriority,
}

impl Installer {
    pub fn from(s: &str) -> Self {
        match s {
            "InstallAll" => Installer::InstallAll,
            "InstallAllDesc" => Installer::InstallAllDesc,
            "InstallRandom" => Installer::InstallRandom,
            "InstallRandomWithPriority" => Installer::InstallRandomWithPriority,
            _ => Installer::InstallAll,
        }
    }
}

impl Into<&str> for Installer {
    fn into(self) -> &'static str {
        match self {
            Installer::InstallAll => "InstallAll",
            Installer::InstallAllDesc => "InstallAllDesc",
            Installer::InstallRandom => "InstallRandom",
            Installer::InstallRandomWithPriority => "InstallRandomWithPriority",
        }
    }
}
