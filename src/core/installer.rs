use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Installer {
    InstallAll,
    InstallRandom,
    InstallRandomWithPriority,
}

// Implementiere die From-Trait, um Installer in Task zu konvertieren
impl Installer {
    pub fn from(s: &str) -> Self {
        match s {
            "InstallAll" => Installer::InstallAll,
            "InstallRandom" => Installer::InstallRandom,
            "InstallRandomWithPriority" => Installer::InstallRandomWithPriority,
            _ => Installer::InstallAll, // Fallback-Wert, wenn der übergebene String nicht erkannt wird
        }
    }
}

// Implementiere die Into-Trait, um Task in Installer zu konvertieren
impl Into<&str> for Installer {
    fn into(self) -> &'static str {
        match self {
            Installer::InstallAll => "InstallAll",
            Installer::InstallRandom => "InstallRandom",
            Installer::InstallRandomWithPriority => "InstallRandomWithPriority",
        }
    }
}
