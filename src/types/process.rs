use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use bx::network::address::Address;
use bx::network::url::Url;
use delegate::delegate;
use serde_json::json;
use tokio::io;
use tokio::process::{ChildStdin, Child, Command};
use tokio::time::{sleep, timeout as wait, Instant};

use crate::{error, log_error, log_info, log_warning};
use crate::config::{CloudConfig, SoftwareName};
use crate::types::service::Service;
use crate::types::{EntityId, ServiceStatus};
use crate::types::task::Task;
use crate::utils::error::*;
use crate::utils::utils::Utils;

pub struct ServiceProcess {
    service: Service,
    shutdown_initiated_by_cloud: bool,
    process: Option<Child>,
    stdin: Option<ChildStdin>,
}


impl ServiceProcess {

    pub fn new(service: Service) -> ServiceProcess {
        ServiceProcess {
            service,
            shutdown_initiated_by_cloud: false,
            process: None,
            stdin: None,
        }
    }

    pub fn create(task: &Task) -> CloudResult<ServiceProcess> {
        Ok(ServiceProcess {
            service: Service::new(task)?,
            shutdown_initiated_by_cloud: false,
            process: None,
            stdin: None,
        })
    }

    pub fn start(&mut self) -> CloudResult<()> {
        let server_file_path = self.get_path_with_server_file()
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

        self.stdin      = child.stdin.take();
        self.process    = Some(child);

        Ok(())
    }


    pub async fn shutdown(&mut self, msg: &str) -> CloudResult<()> {
        if self.service.is_stop() {
            return Ok(());
        }
        self.shutdown_initiated_by_cloud = true;
        self.service.set_status(ServiceStatus::Stopping);
        self.service.save_to_file();

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
        let should_kill = self
            .wait_for_exit_or_kill(timeout)
            .await
            .unwrap_or(true);

        // 3. Kill falls nötig
        if should_kill {
            match self.kill().await {
                Ok(_) => log_info!(5, "Service: [{}] kill", self.service.get_name()),
                Err(_) => log_warning!(2, "Service: [{}] can't kill", self.service.get_name()),
            }
        }

        self.service.set_status(ServiceStatus::Stopped);
        self.service.save_to_file();

        Ok(())
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
        let url = self.service.get_service_url().join("shutdown");
        let timeout = self.service.get_task().get_time_shutdown_before_kill();
        let fut = tokio::spawn(async move { url.post(&body, timeout).await });

        match wait(timeout, fut).await {
            Ok(join_result) => {
                match join_result {
                    Ok(Ok(_)) => {
                        log_info!(6, "Service Stop command successes Send to Service: [{}]", self.service.get_name());
                        Ok(())
                    }
                    Ok(Err(e)) => Err(error!(CantSendShutdownRequest, e)),
                    Err(e) => Err(error!(CantSendShutdownRequest, e)),
                }
            }
            Err(_) => {
                // Timeout
                log_warning!(5,
                    "Shutdown request for [{}] take to long -> Kill Thread",
                    self.service.get_name()
                );
                Err(error!(CantSendShutdownRequest ,"Shutdown Timeout"))
            }
        }
    }

    async fn wait_for_exit_or_kill(
        &mut self,
        timeout: Duration,
    ) -> io::Result<bool> {
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

    pub fn get_service(&self) -> Service {
        self.service.clone()
    }

    pub fn is_shutdown_init(&self) -> bool {
        self.shutdown_initiated_by_cloud
    }

    delegate! {
        to self.service {
            pub fn get_id(&self) -> EntityId;
            pub fn get_name(&self) -> String;
            pub fn get_status(&self) -> ServiceStatus;
            pub fn is_start(&self) -> bool;
            pub fn is_stop(&self) -> bool;
            pub fn is_proxy(&self) -> bool;
            pub fn is_backend_server(&self) -> bool;
            pub fn is_delete(&self) -> bool;
            pub fn get_server_listener(&self) -> Address;
            pub fn get_plugin_listener(&self) -> Address;
            pub fn get_service_url(&self) -> Url;
            pub fn get_software_name(&self) -> SoftwareName;
            pub fn get_path_with_server_file(&self) -> PathBuf;
            pub fn get_path_stdout_file(&self) -> PathBuf;
            pub fn get_path_stderr_file(&self) -> PathBuf;
            pub fn get_path(&self) -> PathBuf;
            pub fn get_start_node(&self) -> String;
            pub fn get_task(&self) -> Task;
            pub fn get_started_at_to_string(&self) -> Option<String>;
            pub fn get_stopped_at_to_string(&self) -> Option<String>;
            pub fn save_to_file(&self);
            pub fn delete_files(&self);

            pub fn set_status(&mut self, status: ServiceStatus);
            pub fn set_server_listener(&mut self, address: Address);
            pub fn set_plugin_listener(&mut self, address: Address);
            pub fn update_current_player(&mut self, count: u32);

            pub fn start_idle_timer(&mut self);

            #[deprecated]
            pub fn install_software(&self) -> CloudResult<()>;

            #[deprecated]
            pub fn install_system_plugin(&self) -> CloudResult<()>;

            #[deprecated]
            pub fn install_software_lib(&self, config: &CloudConfig) -> CloudResult<()>;
        }
    }

}

