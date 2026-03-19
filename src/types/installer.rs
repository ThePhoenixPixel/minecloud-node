use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Installer {
    InstallAll,
    InstallAllDesc,
    InstallRandom,
    InstallRandomWithPriority,
}

impl Installer {
    pub fn from_str(s: &str) -> Self {
        match s {
            "InstallAll" => Installer::InstallAll,
            "InstallAllDesc" => Installer::InstallAllDesc,
            "InstallRandom" => Installer::InstallRandom,
            "InstallRandomWithPriority" => Installer::InstallRandomWithPriority,
            _ => Installer::InstallAll,
        }
    }

    pub fn too_str(value: &Installer) -> &str {
        match value {
            Installer::InstallAll => "InstallAll",
            Installer::InstallAllDesc => "InstallAllDesc",
            Installer::InstallRandom => "InstallRandom",
            Installer::InstallRandomWithPriority => "InstallRandomWithPriority",
        }
    }
}

impl From<&str> for Installer {
    fn from(s: &str) -> Self {
        Installer::from_str(s)
    }
}

impl From<String> for Installer {
    fn from(s: String) -> Self {
        Installer::from_str(s.as_str())
    }
}

impl From<Installer> for String {
    fn from(value: Installer) -> Self {
       Installer::too_str(&value).to_string()
    }
}

impl From<&Installer> for String {
    fn from(value: &Installer) -> Self {
        Installer::too_str(value).to_string()
    }
}
