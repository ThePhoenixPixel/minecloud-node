use bx::network::url::Url;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use strum::IntoEnumIterator;
use tokio::sync::RwLock;

use crate::config::cloud_config::CloudConfig;
use crate::types::{SoftwareLink, SoftwareType};
use crate::{error, log_error, log_info, log_warning};
use crate::utils::error::*;

#[derive(Debug)]
pub struct SoftwareConfig {
    system_config: Arc<CloudConfig>,
    software: HashMap<SoftwareLink, Software>,
}

impl SoftwareConfig {

    /// load the Software from a CloudConfig and return a SoftwareConfig Obj.
    pub fn load(system_config: Arc<CloudConfig>) -> SoftwareConfig {
        let software_path = system_config
            .get_cloud_path()
            .get_system_folder()
            .get_software_config_path();

        let mut software = HashMap::new();

        for software_type in SoftwareType::iter() {
            let type_path = software_path.join(software_type.to_string());

            if !type_path.is_dir() {
                continue;
            }

            let name_entries = match fs::read_dir(&type_path) {
                Ok(e) => e,
                Err(e) => {
                    log_warning!("Cannot read dir {}: {}", type_path.display(), e);
                    continue;
                }
            };

            for name_entry in name_entries.flatten() {
                let name_path = name_entry.path();
                if !name_path.is_dir() {
                    continue;
                }

                let version_entries = match fs::read_dir(&name_path) {
                    Ok(e) => e,
                    Err(e) => {
                        log_warning!("Cannot read dir {}: {}", name_path.display(), e);
                        continue;
                    }
                };

                for version_entry in version_entries.flatten() {
                    let version_path = version_entry.path().join("software.json");
                    if version_path.extension().and_then(|e| e.to_str()) != Some("json") {
                        continue;
                    }

                    Self::load_file(&version_path, &mut software);
                }
            }
        }

        SoftwareConfig {
            system_config,
            software
        }
    }

    fn load_file(
        path: &PathBuf,
        map: &mut HashMap<SoftwareLink, Software>,
    ) {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                log_warning!(2, "Failed to read {}: {}", path.display(), e); return;
            }
        };

        match serde_json::from_str::<Software>(&content) {
            Ok(s) => {
                let link = s.create_link();
                map.insert(link, s);
            }
            Err(e) => log_warning!(2, "Failed to parse {}: {}", path.display(), e),
        }
    }

    #[deprecated]
    pub fn get() -> SoftwareConfig {
        Self::load(Arc::new(CloudConfig::get()))
    }

    #[deprecated]
    pub fn find_software(link: &SoftwareLink) -> Option<Software> {
        Self::load(Arc::new(CloudConfig::get())).software.get(link).cloned()
    }

    /// return all Software
    pub fn get_all(&self) -> &HashMap<SoftwareLink, Software> {
        &self.software
    }

    // Orchestrator — ruft alles in der richtigen Reihenfolge auf
    pub async fn check_and_get(system_config: Arc<CloudConfig>, url: &String) -> CloudResult<SoftwareConfig> {
        let software_path = CloudConfig::get()
            .get_cloud_path()
            .get_system_folder()
            .get_software_config_path();

        let needs_install = !software_path.exists() || SoftwareType::iter().any(|software_type| {
            !software_path.join(software_type.to_string()).exists()
        });

        if needs_install {
            SoftwareConfig::install_configs(&system_config, url).await?;
        }

        let software_config = SoftwareConfig::load(system_config);

        for software in software_config.software.values() {
            software_config.install_server_file(software).await?;
            software_config.install_system_plugin(software).await?;
            software_config.install_libs(software).await?;

        }

        Ok(software_config)
    }

    // Lädt die .json Config-Dateien vom Server
    pub async fn install_configs(system_config: &Arc<CloudConfig>, start_url: &String) -> CloudResult<()> {
        let base_dir = system_config
            .get_cloud_path()
            .get_system_folder()
            .get_software_config_path();

        for software_type in SoftwareType::iter() {
            let type_path = base_dir.join(software_type.to_string());
            if let Err(e) = fs::create_dir_all(&type_path) {
                return Err(error!(CantCreateSoftwareConfigPath, e));
            }
        }

        let index_url = format!("{}config/software/index.json", start_url);
        let index_content = match reqwest::get(&index_url).await {
            Ok(res) => match res.text().await {
                Ok(text) => text,
                Err(e) => return Err(error!(CantFetchSoftwareIndex, e)),
            },
            Err(e) => return Err(error!(CantFetchSoftwareIndex, e)),
        };

        let files: Vec<String> = match serde_json::from_str(&index_content) {
            Ok(f) => f,
            Err(e) => return Err(error!(CantParseSoftwareIndex, e)),
        };

        for file in &files {
            let url = format!("{}config/software/{}", start_url, file);
            let path = base_dir.join(file);

            let mut folder_path = base_dir.join(file);
            folder_path.pop();

            fs::create_dir_all(folder_path)
                .map_err(|e| error!(CantCreateSoftwareConfigPath, e))?;


            match Url::download_file(url.as_str(), &path).await {
                Ok(_) => log_info!("Downloaded software config: {}", file),
                Err(e) => return Err(error!(CantDownloadSoftwareConfig, e)),
            }
        }

        log_info!("Successfully installed {} software configs", files.len());
        Ok(())
    }

    /// install/download the Jar/exe/binary in the Software Path
    pub async fn install_server_file(&self, software: &Software) -> CloudResult<()> {
        let server_path = self.get_software_server_path(&software.create_link());
        fs::create_dir_all(&server_path).map_err(|e| error!(CantCreateSoftwareConfigPath, e))?;

        let software_file_url = software.get_software_file().get_url();
        let jar_path = match Url::extract_extension_from_url(&software_file_url) {
            Some(ext) => server_path.join(format!("{}.{}", software.get_name(), ext)),
            None => server_path.join(software.get_name()),
        };

        if !jar_path.exists() {
            log_info!("Downloading jar {}-{}", software.get_name(), software.get_version());
            match Url::download_file(&software_file_url, &jar_path).await {
                Ok(_) => log_info!("Downloaded jar {}-{}", software.get_name(), software.get_version()),
                Err(e) => {
                    log_error!("Failed to download jar {}-{}: {}", software.get_name(), software.get_version(), e);
                    return Err(error!(CantDownloadSoftwareConfig, e));
                }
            }
        }

        Ok(())
    }

    /// install/download the Plugin in the Software Path
    pub async fn install_system_plugin(&self, software: &Software) -> CloudResult<()> {
        let plugin = software.get_system_plugin();
        if plugin.is_local() { return Ok(()); }

        let plugins_path = self.get_software_plugin_path(&software.create_link());
        fs::create_dir_all(&plugins_path).map_err(|e| error!(CantCreateSoftwareConfigPath, e))?;

        let plugin_path = match Url::extract_extension_from_url(&plugin.get_download()) {
            Some(ext) => plugins_path.join(format!("MineCloud-{}.{}", software.get_name(), ext)),
            None => plugins_path.join(software.get_name()),
        };

        if !plugin_path.exists() {
            log_info!("Downloading plugin for {}-{}", software.get_name(), software.get_version());
            match Url::download_file(plugin.get_download().as_str(), &plugin_path).await {
                Ok(_) => log_info!("Downloaded plugin for {}-{}", software.get_name(), software.get_version()),
                Err(e) => {
                    log_error!("Failed to download plugin for {}-{}: {}", software.get_name(), software.get_version(), e);
                    return Err(error!(CantDownloadSoftwareConfig, e));
                }
            }
        }

        Ok(())
    }

    /// install/download the Lib/dependency in the Software Path
    pub async fn install_libs(&self, software: &Software) -> CloudResult<()> {
        let lib_path = self.get_software_lib_path(&software.create_link());
        fs::create_dir_all(&lib_path).map_err(|e| error!(CantCreateSoftwareConfigPath, e))?;

        for (url_str, lib_file) in software.get_software_lib() {
            let full_path = lib_path.join(lib_file);

            if !full_path.exists() || software.get_software_file().is_auto_update() {
                match Url::download_file(url_str, &full_path).await {
                    Ok(_) => log_info!("Downloaded lib {} to {:?}", url_str, full_path),
                    Err(e) => log_warning!("Failed to download lib {}: {}", url_str, e),
                }
            }
        }

        Ok(())
    }

    pub fn get_software_folder_path(&self, link: &SoftwareLink) -> PathBuf {
        self.system_config
            .get_cloud_path()
            .get_system_folder()
            .get_software_config_path()
            .join(link.get_software_type().to_string())
            .join(link.get_name())
            .join(link.get_version())
    }

    // todo: change fn name to ..... better
    pub fn get_software_file_path(&self, software: &Software) -> PathBuf {
        let software_files_path = CloudConfig::get()
            .get_cloud_path()
            .get_system_folder()
            .get_software_files_folder_path();

        match Url::extract_extension_from_url(&software.get_software_file().get_url()) {
            Some(ext) => software_files_path.join(format!("{}.{}", software.get_name(), ext)),
            None => software_files_path.join(software.get_name()),
        }
    }

    pub fn get_software_server_path(&self, link: &SoftwareLink) -> PathBuf {
        self.get_software_folder_path(link)
    }

    pub fn get_software_plugin_path(&self, link: &SoftwareLink) -> PathBuf {
        self.get_software_folder_path(link).join("plugin")
    }

    pub fn get_software_lib_path(&self, link: &SoftwareLink) -> PathBuf {
        self.get_software_folder_path(link).join("lib")
    }

    /*
    pub fn get_software_lib_path(&self, software: &Software) -> HashMap<String, PathBuf> {
        let software_lib_path = self.system_config
            .get_cloud_path()
            .get_system_folder()
            .get_software_lib_folder_path();

        software.software_lib
            .iter()
            .map(|(url_str, path_str)| {
                let path = software_lib_path
                    .join(self.get_name())
                    .join(path_str);
                (url_str.clone(), path)
            })
            .collect()
    }*/

}

// -----------------------------------------------------------
// SoftwareName — entspricht direkt einer .json Datei
// -----------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct Software {
    name: String,
    typ: SoftwareType,
    version: String,

    software_file: SoftwareFile,
    environment: Environment,
    max_ram: u32,
    ip_path: String,
    port_path: String,
    system_plugin: SystemPlugin,
    software_lib: HashMap<String, String>,
}

impl Software {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_typ(&self) -> &SoftwareType {
        &self.typ
    }

    pub fn get_version(&self) -> &str {
        &self.version
    }

    pub fn get_software_file(&self) -> SoftwareFile {
        self.software_file.clone()
    }

    pub fn get_environment(&self) -> Environment {
        self.environment.clone()
    }

    pub fn get_max_ram(&self) -> u32 {
        self.max_ram
    }

    pub fn get_ip_path(&self) -> String {
        self.ip_path.clone()
    }

    pub fn get_port_path(&self) -> String {
        self.port_path.clone()
    }

    pub fn get_system_plugin(&self) -> SystemPlugin {
        self.system_plugin.clone()
    }

    pub fn create_link(&self) -> SoftwareLink {
        SoftwareLink::new(self.typ.clone(), self.name.clone(), self.version.clone())
    }

    pub fn get_software_lib(&self) -> &HashMap<String, String> {
        &self.software_lib
    }

}

// -----------------------------------------------------------
// Hilfstrukturen (unverändert)
// -----------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct Environment {
    command: String,
    process_args: Vec<String>,
}

impl Environment {
    pub fn get_command(&self) -> String {
        self.command.clone()
    }
    pub fn get_process_args(&self) -> Vec<String> {
        self.process_args.clone()
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct SoftwareFile {
    url: String,
    file_name: String,
    auto_update: bool,
}

impl SoftwareFile {
    pub fn get_url(&self) -> String {
        self.url.clone()
    }
    pub fn get_file_name(&self) -> String {
        self.file_name.clone()
    }
    pub fn is_auto_update(&self) -> bool {
        self.auto_update
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct SystemPlugin {
    local: bool,
    download: String,
    path: String,
}

impl SystemPlugin {
    pub fn is_local(&self) -> bool {
        self.local
    }
    pub fn get_download(&self) -> String {
        self.download.clone()
    }
    pub fn get_path(&self) -> String {
        self.path.clone()
    }
}

pub struct SoftwareConfigRef(Arc<RwLock<SoftwareConfig>>);

impl SoftwareConfigRef {
    pub fn new(software_config: SoftwareConfig) -> SoftwareConfigRef {
        SoftwareConfigRef(Arc::new(RwLock::new(software_config)))
    }

    pub async fn find_software(&self, link: &SoftwareLink) -> Option<Software> {
        let config = self.0.read().await;
        config.software.get(link).cloned()
    }

    pub async fn get_all(&self) -> HashMap<SoftwareLink, Software> {
        let config = self.0.read().await;
        config.get_all().clone()
    }
}

impl Clone for SoftwareConfigRef {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}