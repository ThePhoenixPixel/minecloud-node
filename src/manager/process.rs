use std::collections::HashMap;
use std::fs::File;
use std::process::Stdio;
use serde_json::json;
use tokio::io;
use tokio::process::{ChildStdin, Child, Command};
use tokio::time::timeout as wait;

use crate::{error, log_error, log_info, log_warning};
use crate::types::service::Service;
use crate::types::ServiceStatus;
use crate::types::task::Task;
use crate::utils::error::cloud_error::CloudError;
use crate::utils::error::error_kind::CloudErrorKind::*;
use crate::utils::utils::Utils;

pub struct ServiceProcess {
    service: Service,
    process: Option<Child>,
    stdin: Option<ChildStdin>,
}


impl ServiceProcess {

    pub fn new(service: Service) -> ServiceProcess {
        ServiceProcess {
            service,
            process: None,
            stdin: None,
        }
    }

    pub fn create(task: &Task) -> Result<ServiceProcess, CloudError> {
        Ok(ServiceProcess {
            service: Service::new(task)?,
            process: None,
            stdin: None,
        })
    }

    pub async fn start_async(mut self) -> Result<Self, CloudError> {
        self.start()?; // falls intern sync
        Ok(self)
    }

    pub fn start(&mut self) -> Result<(), CloudError> {
        let service = self.get_service();

        let server_file_path = service.get_path_with_server_file()
            .to_str()
            .ok_or(error!(CantConvertServerFilePathToString))?
            .to_string();

        let software_name = service.get_software_name();
        let mut placeholders = HashMap::new();
        placeholders.insert("ip", service.get_server_listener().get_ip().to_string());
        placeholders.insert("port", service.get_server_listener().get_port().to_string());
        placeholders.insert("max_ram", software_name.get_max_ram().to_string());
        placeholders.insert("server_file", server_file_path);

        let process_args = Utils::replace_placeholders(
            software_name.get_environment().get_process_args(),
            &placeholders,
        );

        let stdout_file = File::create(service.get_path_stdout_file())
            .map_err(|e| error!(CantCreateSTDOUTFile, e))?;
        let stderr_file = File::create(service.get_path_stderr_file())
            .map_err(|e| error!(CantCreateSTDERRFile, e))?;

        let mut child = Command::new(software_name.get_environment().get_command())
            .args(&process_args)
            .current_dir(service.get_path())
            .stdin(Stdio::piped())
            .stdout(Stdio::from(stdout_file))
            .stderr(Stdio::from(stderr_file))
            .spawn()
            .map_err(|e| error!(CantStartServer, e))?;

        self.stdin      = child.stdin.take();
        self.process    = Some(child);

        Ok(())
    }


    pub async fn shutdown(&mut self, msg: &str) -> Result<(), CloudError> {
        if self.service.is_stop() {
            return Ok(());
        }
        self.service.set_status(ServiceStatus::Stopping);
        let mut should_kill = true;

        match self.send_stop(msg).await {
            Ok(_) => {
                if let Some(child) = self.get_process_mut() {
                    match child.try_wait() {
                        Ok(Some(_status)) => should_kill = false,
                        Ok(None) => should_kill = true,
                        Err(_) => should_kill = true,
                    }
                } else {
                    should_kill = false;
                }
            }
            Err(e) => {

                log_error!(
                    "Stop command nicht senden an {} \n Error: {}",
                    self.service.get_name(),
                    e.to_string()
                );

                if self.get_process().is_none() {
                    self.service.set_status(ServiceStatus::Stopped);
                    self.service.delete_files();
                }
            }
        }

        if should_kill || self.process.is_some() {
            match self.kill().await {
                Ok(..) => log_info!(5, "Service: [{}] kill", self.service.get_name()),
                Err(..) => log_warning!(2, "Service: [{}] can't kill", self.service.get_name()),
            }
        }
        self.service.set_status(ServiceStatus::Stopped);
        self.service.save_to_file();
        Ok(())
    }

    pub async fn kill(&mut self) -> io::Result<()> {
        if let Some(mut child) = self.process.take() {
            child.kill().await?;
            child.wait().await?;
        }
        Ok(())
    }

    async fn send_stop(&mut self, msg: &str) -> Result<(), CloudError> {
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

    pub fn clone_without_process(&self) -> ServiceProcess {
        ServiceProcess {
            service: self.service.clone(),
            process: None,
            stdin: None,
        }
    }

    pub fn get_service(&self) -> &Service {
        &self.service
    }

    pub fn get_service_mut(&mut self) -> &mut Service {
        &mut self.service
    }

    pub fn extract_process(self) -> Option<Child> {
        self.process
    }

    pub fn get_process(&self) -> Option<&Child> {
        self.process.as_ref()
    }

    pub fn get_process_mut(&mut self) -> Option<&mut Child> {
        self.process.as_mut()
    }

    pub fn set_process(&mut self, process: Option<Child>) {
        self.process = process;
    }

    pub fn is_start(&self) -> bool {
        let status = self.service.get_status();
        status == ServiceStatus::Starting || status == ServiceStatus::Running
    }

    pub fn is_stop(&self) -> bool {
        let status = self.service.get_status();
        status == ServiceStatus::Stopping || status == ServiceStatus::Stopped
    }

    pub fn is_failed(&self) -> bool {
        self.service.get_status() == ServiceStatus::Failed
    }
}

