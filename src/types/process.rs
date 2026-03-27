use bx::network::address::Address;
use bx::network::url::{Url, UrlSchema};
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
use tokio::io;
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use tokio::time::{Instant, sleep, timeout as wait};

use crate::config::Software;
use crate::types::service::Service;
use crate::types::{EntityId, ServiceConfig, ServiceStatus};
use crate::utils::error::*;
use crate::utils::utils::Utils;
use crate::{error, log_error, log_info, log_warning};

pub struct ServiceProcess {
    service: Service,
    path: PathBuf,
    shutdown_initiated_by_cloud: bool,
    process: Option<Child>,
    stdin: Option<ChildStdin>,
}

pub struct ServiceProcessRef(Arc<RwLock<ServiceProcess>>);

impl ServiceProcess {
    // software wird vom ServiceManager übergeben — er hat die SoftwareConfig
    pub fn start(&mut self, software: &Software) -> CloudResult<()> {
        let server_file_path = self.path
            .join(software.get_software_file().get_file_name())
            .to_str()
            .ok_or(error!(CantConvertServerFilePathToString))?
            .to_string();

        let mut placeholders = HashMap::new();
        placeholders.insert("ip", self.get_server_listener().get_ip().to_string());
        placeholders.insert("port", self.get_server_listener().get_port().to_string());
        placeholders.insert("max_ram", software.get_max_ram().to_string());
        placeholders.insert("server_file", server_file_path);

        let process_args = Utils::replace_placeholders(
            software.get_environment().get_process_args(),
            &placeholders,
        );

        let stdout_file = File::create(self.get_path_stdout_file())
            .map_err(|e| error!(CantCreateSTDOUTFile, e))?;
        let stderr_file = File::create(self.get_path_stderr_file())
            .map_err(|e| error!(CantCreateSTDERRFile, e))?;

        let mut child = Command::new(software.get_environment().get_command())
            .args(&process_args)
            .current_dir(&self.path)
            .stdin(Stdio::piped())
            .stdout(Stdio::from(stdout_file))
            .stderr(Stdio::from(stderr_file))
            .spawn()
            .map_err(|e| error!(CantStartServer, e))?;

        self.stdin = child.stdin.take();
        self.process = Some(child);

        Ok(())
    }

    pub async fn shutdown(&mut self, msg: &str, timeout: Duration) {
        self.shutdown_initiated_by_cloud = true;

        if let Err(e) = self.send_stop(msg, timeout).await {
            log_error!("Stop command failed for {}: {}", self.service.get_name(), e);
        }

        let should_kill = self.wait_for_exit_or_kill(timeout).await.unwrap_or(true);

        if should_kill {
            match self.kill().await {
                Ok(_) => log_info!(5, "Service [{}] killed", self.service.get_name()),
                Err(_) => log_warning!(2, "Service [{}] can't kill", self.service.get_name()),
            }
        }
    }

    pub async fn kill(&mut self) -> io::Result<()> {
        if let Some(mut child) = self.process.take() {
            child.kill().await?;
            child.wait().await?;
        }
        Ok(())
    }

    async fn send_stop(&mut self, msg: &str, timeout: Duration) -> CloudResult<()> {
        let body = json!({ "msg": msg });
        let url = self.get_service_url().join("shutdown");
        let fut = tokio::spawn(async move { url.post(&body, timeout).await });

        match wait(timeout, fut).await {
            Ok(Ok(Ok(_))) => {
                log_info!(6, "Stop command sent to [{}]", self.service.get_name());
                Ok(())
            }
            Ok(Ok(Err(e))) => Err(error!(CantSendShutdownRequest, e)),
            Ok(Err(e)) => Err(error!(CantSendShutdownRequest, e)),
            Err(_) => {
                log_warning!(5, "Shutdown timeout for [{}]", self.service.get_name());
                Err(error!(CantSendShutdownRequest, "Shutdown Timeout"))
            }
        }
    }

    async fn wait_for_exit_or_kill(&mut self, timeout: Duration) -> io::Result<bool> {
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(child) = self.process.as_mut() {
                match child.try_wait()? {
                    Some(_) => return Ok(false),
                    None => {}
                }
            } else {
                return Ok(false);
            }

            if Instant::now() >= deadline { return Ok(true); }
            sleep(Duration::from_millis(100)).await;
        }
    }

    pub fn get_service(&self) -> &Service { &self.service }
    pub fn is_shutdown_init(&self) -> bool { self.shutdown_initiated_by_cloud }

    pub fn get_service_url(&self) -> Url {
        Url::new(UrlSchema::Http, self.get_plugin_listener(), "cloud/service").join(self.get_name())
    }

    pub fn get_path(&self) -> &PathBuf { &self.path }

    pub fn get_path_with_service_config(&self) -> PathBuf { self.path.join(".minecloud") }
    pub fn get_path_with_service_file(&self) -> PathBuf {
        self.get_path_with_service_config().join("service_config.json")
    }
    pub fn get_path_stdout_file(&self) -> PathBuf {
        self.get_path_with_service_config().join("server_stdout.log")
    }
    pub fn get_path_stdin_file(&self) -> PathBuf {
        self.get_path_with_service_config().join("server_stdin.log")
    }
    pub fn get_path_stderr_file(&self) -> PathBuf {
        self.get_path_with_service_config().join("server_stderr.log")
    }

    pub fn set_status(&mut self, status: ServiceStatus) {
        self.service.set_status(status);
        self.save_to_file();
    }

    pub fn save_to_file(&self) {
        let path = self.get_path_with_service_config();
        if fs::create_dir_all(&path).is_err() {
            log_error!("Can't create service config dir");
            return;
        }
        if let Ok(serialized) = serde_json::to_string_pretty(self.get_service()) {
            if let Ok(mut file) = File::create(self.get_path_with_service_file()) {
                file.write_all(serialized.as_bytes())
                    .expect("Error saving service config");
            }
        }
    }

    pub fn delete_files(&self) {
        if fs::remove_dir_all(self.get_path()).is_err() {
            log_warning!("Service [{}] folder can't delete", self.get_name());
        }
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
            pub fn get_task_name(&self) -> &str;
            pub fn get_config(&self) -> &ServiceConfig;
            pub fn is_proxy(&self) -> bool;
            pub fn is_backend_server(&self) -> bool;
            pub fn is_start(&self) -> bool;
            pub fn is_running(&self) -> bool;
            pub fn is_stop(&self) -> bool;
            pub fn is_local_node(&self, node_name: &str) -> bool;

            pub fn set_server_listener(&mut self, address: Address);
            pub fn set_plugin_listener(&mut self, address: Address);
            pub fn set_cloud_listener(&mut self, address: Address);
            pub fn set_current_player(&mut self, count: u32);
            pub fn start_idle_timer(&mut self);
        }
    }
}

impl ServiceProcessRef {
    pub fn new(service: Service, path: PathBuf) -> Self {
        Self(Arc::new(RwLock::new(ServiceProcess {
            service,
            path,
            shutdown_initiated_by_cloud: false,
            process: None,
            stdin: None,
        })))
    }

    pub async fn read(&self) -> RwLockReadGuard<'_, ServiceProcess> { self.0.read().await }
    pub async fn write(&self) -> RwLockWriteGuard<'_, ServiceProcess> { self.0.write().await }
    pub fn ptr_eq(&self, other: &ServiceProcessRef) -> bool { Arc::ptr_eq(&self.0, &other.0) }

    pub async fn get_id(&self) -> EntityId { self.0.read().await.get_id().clone() }
    pub async fn get_name(&self) -> String { self.0.read().await.get_name().to_string() }
    pub async fn is_start(&self) -> bool { self.0.read().await.is_start() }
    pub async fn is_proxy(&self) -> bool { self.0.read().await.is_proxy() }
}

impl Clone for ServiceProcessRef {
    fn clone(&self) -> Self { Self(self.0.clone()) }
}