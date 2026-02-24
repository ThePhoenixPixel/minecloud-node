use bx::network::address::Address;
use bx::network::url::{Url, UrlSchema};
use bx::path::Directory;
use chrono::NaiveDateTime;
use delegate::delegate;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use database_manager::DatabaseController;
use tokio::io;
use tokio::process::{Child, ChildStdin, Command};
use tokio::time::{Instant, sleep, timeout as wait};

use crate::config::{CloudConfig, SoftwareName};
use crate::types::service::Service;
use crate::types::task::Task;
use crate::types::{EntityId, ServiceConfig, ServiceStatus};
use crate::utils::error::*;
use crate::utils::utils::Utils;
use crate::{error, log_error, log_info, log_warning};
use crate::database::table::TableServices;

pub struct ServiceProcess {
    service: Service,
    path: PathBuf, // Path -> ~/(temp/static)/servicename(zb. Lobby-1)/
    shutdown_initiated_by_cloud: bool,
    process: Option<Child>,
    stdin: Option<ChildStdin>,
}

impl ServiceProcess {
    pub fn new(service: Service, path: PathBuf) -> ServiceProcess {
        ServiceProcess {
            service,
            path,
            shutdown_initiated_by_cloud: false,
            process: None,
            stdin: None,
        }
    }

    pub fn start(&mut self) -> CloudResult<()> {
        let server_file_path = self
            .get_path_with_server_file()
            .to_str()
            .ok_or(error!(CantConvertServerFilePathToString))?
            .to_string();

        let software_name = self.get_software_name();
        let mut placeholders = HashMap::new();
        placeholders.insert("ip", self.get_server_listener().get_ip().to_string());
        placeholders.insert("port", self.get_server_listener().get_port().to_string());
        placeholders.insert("max_ram", software_name.get_max_ram().to_string());
        placeholders.insert("server_file", server_file_path);

        let process_args = Utils::replace_placeholders(
            software_name.get_environment().get_process_args(),
            &placeholders,
        );

        let stdout_file = File::create(self.get_path_stdout_file())
            .map_err(|e| error!(CantCreateSTDOUTFile, e))?;
        let stderr_file = File::create(self.get_path_stderr_file())
            .map_err(|e| error!(CantCreateSTDERRFile, e))?;

        let mut child = Command::new(software_name.get_environment().get_command())
            .args(&process_args)
            .current_dir(self.get_path())
            .stdin(Stdio::piped())
            .stdout(Stdio::from(stdout_file))
            .stderr(Stdio::from(stderr_file))
            .spawn()
            .map_err(|e| error!(CantStartServer, e))?;

        self.stdin = child.stdin.take();
        self.process = Some(child);

        Ok(())
    }

    pub async fn shutdown(&mut self, msg: &str) {
        if self.service.is_stop() {
            return;
        }
        self.shutdown_initiated_by_cloud = true;
        self.service.set_status(ServiceStatus::Stopping);
        self.save_to_file();

        let timeout = self.service.get_task().get_time_shutdown_before_kill();

        // 1. Stop senden
        if let Err(e) = self.send_stop(msg).await {
            log_error!(
                "Stop command nicht senden an {} \n Error: {}",
                self.service.get_name(),
                e.to_string()
            );
        }

        // 2. Warten oder Kill entscheiden
        let should_kill = self.wait_for_exit_or_kill(timeout).await.unwrap_or(true);

        // 3. Kill falls nötig
        if should_kill {
            match self.kill().await {
                Ok(_) => log_info!(5, "Service: [{}] kill", self.service.get_name()),
                Err(_) => log_warning!(2, "Service: [{}] can't kill", self.service.get_name()),
            }
        }

        self.service.set_status(ServiceStatus::Stopped);
        self.save_to_file();
    }

    async fn kill(&mut self) -> io::Result<()> {
        if let Some(mut child) = self.process.take() {
            child.kill().await?;
            child.wait().await?;
        }
        Ok(())
    }

    async fn send_stop(&mut self, msg: &str) -> CloudResult<()> {
        let body = json!({ "msg": msg });
        let url = self.get_service_url().join("shutdown");
        let timeout = self.service.get_task().get_time_shutdown_before_kill();
        let fut = tokio::spawn(async move { url.post(&body, timeout).await });

        match wait(timeout, fut).await {
            Ok(join_result) => match join_result {
                Ok(Ok(_)) => {
                    log_info!(
                        6,
                        "Service Stop command successes Send to Service: [{}]",
                        self.service.get_name()
                    );
                    Ok(())
                }
                Ok(Err(e)) => Err(error!(CantSendShutdownRequest, e)),
                Err(e) => Err(error!(CantSendShutdownRequest, e)),
            },
            Err(_) => {
                // Timeout
                log_warning!(
                    5,
                    "Shutdown request for [{}] take to long -> Kill Thread",
                    self.service.get_name()
                );
                Err(error!(CantSendShutdownRequest, "Shutdown Timeout"))
            }
        }
    }

    async fn wait_for_exit_or_kill(&mut self, timeout: Duration) -> io::Result<bool> {
        let deadline = Instant::now() + timeout;

        loop {
            if let Some(child) = self.process.as_mut() {
                match child.try_wait()? {
                    Some(_status) => {
                        return Ok(false);
                    }
                    None => {
                        // process is running
                    }
                }
            } else {
                return Ok(false);
            }

            if Instant::now() >= deadline {
                return Ok(true);
            }

            sleep(Duration::from_millis(100)).await;
        }
    }

    pub fn get_service(&self) -> &Service {
        &self.service
    }

    pub fn is_shutdown_init(&self) -> bool {
        self.shutdown_initiated_by_cloud
    }

    pub fn get_service_url(&self) -> Url {
        Url::new(UrlSchema::Http, self.get_plugin_listener(), "cloud/service").join(self.get_name())
    }

    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }

    pub fn get_path_with_service_config(&self) -> PathBuf {
        self.path.join(".minecloud")
    }

    pub fn get_path_with_service_file(&self) -> PathBuf {
        self.get_path_with_service_config()
            .join("service_config.json")
    }

    pub fn get_path_with_server_file(&self) -> PathBuf {
        self.get_path()
            .join(self.get_task().get_software().get_server_file_name())
    }

    pub fn get_path_stdout_file(&self) -> PathBuf {
        self.get_path_with_service_config()
            .join("server_stdout.log")
    }

    pub fn get_path_stdin_file(&self) -> PathBuf {
        self.get_path_with_service_config()
            .join("server_stdin.log")
    }

    pub fn get_path_stderr_file(&self) -> PathBuf {
        self.get_path_with_service_config()
            .join("server_stderr.log")
    }

    pub fn set_status(&mut self, status: ServiceStatus) {
        self.service.set_status(status);
        self.save_to_file();
    }

    pub fn save_to_file(&self) {
        let path = self.get_path_with_service_config();
        if fs::create_dir_all(&path).is_err() {
            log_error!("Can't create service file in 'save_to_file'");
            return;
        }

        if File::create(self.get_path_with_service_file()).is_err() {
            log_error!("Error by create to service config file");
            return;
        }

        if let Ok(serialized) = serde_json::to_string_pretty(self.get_service()) {
            if let Ok(mut file) = File::create(self.get_path_with_service_file()) {
                file.write_all(serialized.as_bytes())
                    .expect("Error by save the service config file");
            }
        }
    }

    pub fn delete_files(&self) {
        if fs::remove_dir_all(self.get_path()).is_err() {
            log_warning!("Service | {} | folder can't delete", self.get_name());
        }
    }

    #[deprecated]
    pub fn install_software(&self) -> Result<(), CloudError> {
        let target_path = self
            .get_path()
            .join(&self.get_task().get_software().get_server_file_name());
        let software_path = self.get_task().get_software().get_software_file_path();

        fs::copy(&software_path, &target_path).map_err(|e| error!(CantCopySoftware, e))?;
        Ok(())
    }

    #[deprecated]
    pub fn install_system_plugin(&self) -> Result<(), CloudError> {
        let software = self.get_software_name();
        let system_plugin_path = self.get_task().get_software().get_system_plugin_path();
        let mut target_path = self
            .get_path()
            .join(&software.get_system_plugin().get_path());

        if !target_path.exists() {
            fs::create_dir_all(&target_path).map_err(|e| error!(CantCreateSystemPluginPath, e))?;
        }

        target_path.push(self.get_task().get_software().get_system_plugin_name());

        if !system_plugin_path.exists() {
            return Err(error!(CantFindSystemPlugin));
        }

        match fs::copy(system_plugin_path, target_path) {
            Ok(_) => {
                log_info!("Successfully install the System Plugin");
                Ok(())
            }
            Err(e) => Err(error!(CantCopySystemPlugin, e)),
        }
    }

    #[deprecated]
    pub fn install_software_lib(&self, config: &CloudConfig) -> Result<(), CloudError> {
        let software_lib_path = config
            .get_cloud_path()
            .get_system_folder()
            .get_software_lib_folder_path()
            .join(self.get_task().get_software().get_software_type())
            .join(self.get_task().get_software().get_name());

        Directory::copy_folder_contents(&software_lib_path, &self.get_path())
            .map_err(|e| error!(Internal, e))
    }

    delegate! {
        to self.service {
            pub fn get_id(&self) -> &EntityId;
            pub fn get_name(&self) -> &str;
            pub fn get_status(&self) -> ServiceStatus;
            pub fn get_parent_node(&self) -> &str;
            pub fn get_current_players(&self) -> u32;
            pub fn get_started_at(&self) -> Option<NaiveDateTime>;
            pub fn get_stopped_at(&self) -> Option<NaiveDateTime>;
            pub fn get_idle_since(&self) -> Option<NaiveDateTime>;
            pub fn get_server_listener(&self) -> &Address;
            pub fn get_plugin_listener(&self) -> &Address;
            pub fn get_cloud_listener(&self) -> &Address;
            pub fn get_task(&self) -> &Task;
            pub fn is_start(&self) -> bool;
            pub fn is_stop(&self) -> bool;
            pub fn get_config(&self) -> &ServiceConfig;

            pub fn set_server_listener(&mut self, address: Address);
            pub fn set_plugin_listener(&mut self, address: Address);
            pub fn set_cloud_listener(&mut self, address: Address);
            pub fn set_current_player(&mut self, count: u32);
            pub fn start_idle_timer(&mut self);


            #[deprecated]
            pub fn get_started_at_to_string(&self) -> Option<String>;
            #[deprecated]
            pub fn get_stopped_at_to_string(&self) -> Option<String>;
            #[deprecated]
            pub fn get_software_name(&self) -> SoftwareName;
            #[deprecated]
            pub fn is_proxy(&self) -> bool;
            #[deprecated]
            pub fn is_backend_server(&self) -> bool;
            #[deprecated]
            pub fn is_local(&self) -> bool;
        }
    }
}
