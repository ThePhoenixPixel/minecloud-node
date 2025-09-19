use bx::network::address::Address;
use bx::network::url::{Url, UrlSchema};
use bx::path::Directory;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::{fs, io};
use std::fs::{File, read_to_string};
use std::io::{Error, ErrorKind, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::timeout;
use uuid::Uuid;

use crate::cloud::Cloud;
use crate::core::services_local::LocalServices;
use crate::core::task::Task;
use crate::node_api::node_service::ServiceInfoResponse;
use crate::sys_config::cloud_config::CloudConfig;
use crate::sys_config::software_config::SoftwareName;
use crate::utils::logger::Logger;
use crate::utils::service_status::ServiceStatus;
use crate::utils::utils::Utils;
use crate::{log_error, log_info, log_warning};

#[derive(Serialize)]
struct RegisterServerData {
    register_server: ServiceInfoResponse,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Service {
    id: Uuid,
    name: String,
    status: ServiceStatus,
    start_node: String,
    start_time: DateTime<Local>,
    server_address: Address,
    plugin_listener: Address,
    cloud_listener: Address,
    task: Task,

    #[serde(skip)]
    process: Option<Child>,
}

impl Service {
    pub fn new_local(task: &Task) -> Result<Service, Error> {
        let server_address = Address::new(
            &CloudConfig::get().get_server_host(),
            &Address::find_next_port(&mut Address::new(
                &CloudConfig::get().get_server_host(),
                &task.get_start_port(),
            )),
        );
        let service_path = match task.prepared_to_service() {
            Ok(path) => path,
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("Es kann kein neuer Service erstellt werden \n {}", e),
                ));
            }
        };
        let service = Service {
            id: Uuid::new_v4(),
            name: Directory::get_last_folder_name(&service_path),
            status: ServiceStatus::Stop,
            start_node: CloudConfig::get().get_name(),
            start_time: Local::now(),
            server_address,
            plugin_listener: Address::get_local_ipv4(),
            cloud_listener: CloudConfig::get().get_node_host(),
            task: task.clone(),

            process: None,
        };
        service.save_to_file();
        Ok(service)
    }

    pub fn clone_without_process(&self) -> Self {
        Self {
            id: self.id.clone(),
            name: self.name.clone(),
            status: self.status.clone(),
            start_node: self.start_node.clone(),
            start_time: self.start_time.clone(),
            server_address: self.server_address.clone(),
            plugin_listener: self.plugin_listener.clone(),
            cloud_listener: self.cloud_listener.clone(),
            task: self.task.clone(),
            process: None,
        }
    }

    pub fn update(&mut self, s: &Service) {
        self.name = s.name.clone();
        self.status = s.status.clone();
        self.start_node = s.start_node.clone();
        self.start_time = s.start_time.clone();
        self.server_address = s.server_address.clone();
        self.plugin_listener = s.plugin_listener.clone();
        self.cloud_listener = s.cloud_listener.clone();
        self.task = s.task.clone();
    }

    pub fn new_id(&mut self) {
        self.id = Uuid::new_v4();
    }
    pub fn get_id(&self) -> Uuid {
        self.id.clone()
    }

    pub fn is_local(&self) -> bool {
        self.start_node == CloudConfig::get().get_name()
    }

    pub fn get_start_node(&self) -> String {
        self.start_node.to_string()
    }

    pub fn set_start_node(&mut self, node: &String) {
        self.start_node = node.to_string();
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn set_name(&mut self, name: &String) {
        self.name = name.clone();
        self.save_to_file();
    }

    pub fn get_status(&self) -> ServiceStatus {
        self.status.clone()
    }

    pub fn set_status(&mut self, status: ServiceStatus) {
        self.status = status;
    }

    pub fn get_time_to_string(&self) -> String {
        self.start_time.to_string()
    }

    pub fn set_time(&mut self) {
        self.start_time = Local::now();
        self.save_to_file();
    }

    pub fn get_task(&self) -> Task {
        self.task.clone()
    }

    pub fn get_software_name(&self) -> SoftwareName {
        self.get_task().get_software().get_software_name()
    }

    pub fn get_plugin_listener(&self) -> Address {
        self.plugin_listener.clone()
    }

    pub fn set_plugin_listener(&mut self, address: &Address) {
        self.plugin_listener = address.clone();
        self.save_to_file();
    }

    pub fn get_cloud_listener(&self) -> Address {
        self.cloud_listener.clone()
    }

    pub fn set_cloud_listener(&mut self, address: Address) {
        self.cloud_listener = address;
        self.save_to_file()
    }

    pub fn get_server_address(&self) -> Address {
        self.server_address.clone()
    }

    pub fn set_server_address(&mut self) -> Result<(), Error> {
        let address = self.find_free_server_address();

        let software_name = self.get_software_name();
        let path = self.get_path();

        // replace ip1
        let path_ip = path.join(software_name.get_ip_path());

        if !path_ip.exists() {
            log_error!(
                "Die config datei für die Ip des Servers konnte nicht gefunden werden {:?}",
                &path_ip
            );
            return Err(Error::new(
                ErrorKind::Other,
                "Die config datei für die Ip des Servers konnte nicht gefunden werden",
            ));
        }

        let file_content_ip = read_to_string(&path_ip)?;
        let edit_file_ip = file_content_ip.replace("%ip%", &*address.get_ip());
        fs::write(&path_ip, edit_file_ip)?;

        // replace port
        let path_port = path.join(software_name.get_port_path());

        if !path_port.exists() {
            log_error!(
                "Die config datei für den Port des Servers konnte nicht gefunden werden {:?}",
                &path_ip
            );
            return Err(Error::new(
                ErrorKind::Other,
                "Die config datei für den Port des Servers konnte nicht gefunden werden",
            ));
        }

        let file_content_port = read_to_string(&path_port)?;
        let edit_file_port =
            file_content_port.replace("%port%", address.get_port().to_string().as_str());
        fs::write(&path_port, edit_file_port)?;

        self.server_address = address;

        Ok(())
    }

    pub fn kill(&mut self) -> io::Result<()> {
        if let Some(mut child) = self.process.take() {
            child.kill()?;
            child.wait()?;
            self.set_status(ServiceStatus::Stop);
        }
        Ok(())
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

    pub async fn shutdown(&mut self, msg: &str) {
        if self.is_stop() {
            return;
        }

        if self.is_local() {
            let mut should_kill = true;

            // Stop-Befehl senden
            match self.send_stop(msg).await {
                Ok(_) => {
                    // Todo: Aus task lesen als wert 'time shutdown before Kill'
                    tokio::time::sleep(Duration::from_secs(5)).await;

                    if let Some(child) = self.get_process_mut() {
                        match child.try_wait() {
                            Ok(Some(_status)) => should_kill = false, // Prozess ist schon beendet
                            Ok(None) => should_kill = true,           // läuft noch
                            Err(_) => should_kill = true,
                        }
                    } else {
                        should_kill = false;
                    }
                }
                Err(e) => {
                    log_error!(
                    "Stop command nicht senden an {} \n Error: {}",
                    self.get_name(),
                    e.to_string()
                );

                    if self.get_process().is_none() {
                        self.set_status(ServiceStatus::Stop);
                        self.delete_files();
                        return;
                    }
                }
            }

            if should_kill {
                match self.kill() {
                    Ok(..) => log_info!("Service: {} wurde gekillt", self.get_name()),
                    Err(..) => log_warning!("Service konnte nicht gekillt werden"),
                }
            }

            self.delete_files();

            self.set_status(ServiceStatus::Stop);
        } else {
            // TODO: Remote/Cluster shutdown
        }
    }


    pub fn delete_files(&self) {
        if !self.get_task().is_static_service() && self.get_task().is_delete_on_stop() {
            if fs::remove_dir_all(self.get_path()).is_err() {
                log_warning!("Service | {} | folder can't delete", self.get_name());
            }
        }
    }

    async fn send_stop(&mut self, msg: &str) -> Result<(), Error> {
        let body = json!({ "msg": msg });
        let url = self.get_service_url().join("shutdown");

        // Spawn einen "Thread" in Tokio
        let fut = tokio::spawn(async move { url.post(&body, Duration::from_secs(3)).await });

        // Warte maximal 3 Sekunden
        match timeout(Duration::from_secs(3), fut).await {
            Ok(join_result) => {
                // Prüfen, ob der Thread erfolgreich lief
                match join_result {
                    Ok(Ok(_)) => {
                        log_info!("Service erfolgreich heruntergefahren {}", self.get_name());
                        self.set_status(ServiceStatus::Stop);
                        Ok(())
                    }
                    Ok(Err(e)) => Err(Error::new(ErrorKind::Other, e.to_string())),
                    Err(e) => Err(Error::new(ErrorKind::Other, e.to_string())),
                }
            }
            Err(_) => {
                // Timeout überschritten -> Thread abbrechen
                log_warning!(
                    "Shutdown request für {} hat zu lange gedauert. Kill Thread.",
                    self.get_name()
                );
                // Tokio-Task wird automatisch abgebrochen, wenn Timeout überschritten
                Err(Error::new(ErrorKind::TimedOut, "Shutdown Timeout"))
            }
        }
    }

    pub fn find_free_server_address(&self) -> Address {
        let ports = LocalServices::get_bind_ports_from_file();
        let port = self.get_task().get_start_port();
        let server_host = CloudConfig::get().get_server_host();
        Address::new(&server_host, &find_port(ports, port, &server_host))
    }

    pub fn find_free_plugin_address(&self) -> Address {
        let ports = LocalServices::get_bind_ports_from_file();
        let port = self.get_server_address().get_port() + 1;
        let server_host = CloudConfig::get().get_server_host();
        Address::new(&server_host, &find_port(ports, port, &server_host))
    }

    pub fn get_path(&self) -> PathBuf {
        self.get_task().get_service_path().join(self.get_name())
    }

    pub fn get_path_with_server_file(&self) -> PathBuf {
        self.get_path()
            .join(self.get_task().get_software().get_server_file_name())
    }

    pub fn find_new_free_plugin_listener(&mut self) {
        let address = self.find_free_plugin_address();
        self.set_plugin_listener(&address);
        self.save_to_file()
    }

    pub fn get_path_with_service_config(&self) -> PathBuf {
        self.get_path().join(".minecloud")
    }

    pub fn get_path_with_service_file(&self) -> PathBuf {
        self.get_path_with_service_config()
            .join("service_config.json")
    }

    pub fn get_from_path(path: &mut PathBuf) -> Option<Service> {
        //path -> /service/temp/Lobby-1/
        path.push(".minecloud");
        path.push("service_config.json");
        if let Ok(file_content) = read_to_string(path) {
            if let Ok(service) = serde_json::from_str(&file_content) {
                Some(service)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_path_stdout_file(&self) -> PathBuf {
        self.get_path_with_service_config()
            .join("server_stdout.log")
    }

    pub fn get_path_stdin_file(&self) -> PathBuf {
        self.get_path_with_service_config().join("server_stdin.log")
    }

    pub fn get_path_stderr_file(&self) -> PathBuf {
        self.get_path_with_service_config()
            .join("server_stderr.log")
    }

    pub fn save_to_file(&self) {
        let path = self.get_path_with_service_config();
        fs::create_dir_all(&path).expect("Cant create Service File in 'save_to_file'");
        if File::create(self.get_path_with_service_file()).is_err() {
            log_error!("Error by create to service config file");
            return;
        }

        if let Ok(serialized) = serde_json::to_string_pretty(&self) {
            if let Ok(mut file) = File::create(self.get_path_with_service_file()) {
                file.write_all(serialized.as_bytes())
                    .expect("Error by save the service config file");
            }
        }
    }

    pub fn is_start(&self) -> bool {
        self.get_status().is_start()
    }
    pub fn is_prepare(&self) -> bool {
        self.get_status().is_prepare()
    }
    pub fn is_stop(&self) -> bool {
        self.get_status().is_stop()
    }

    // wie viele services muss ich noch starten???
    pub fn get_starts_service_from_task(task: &Task) -> u64 {
        let service_path = task.get_service_path();
        let mut start_service: u64 = 0;
        let files_name = Directory::get_files_name_from_path(&service_path);

        for file_name in files_name {
            let mut current_service_path = service_path.clone();
            if file_name.starts_with(&task.get_name()) {
                current_service_path.push(file_name);

                if Service::is_service_start_or_prepare(&mut current_service_path) {
                    start_service += 1;
                }
            }
        }
        start_service
    }

    pub fn is_service_start_or_prepare(path: &mut PathBuf) -> bool {
        match Service::get_from_path(path) {
            Some(service) => service.is_start() || service.is_prepare(),
            None => false,
        }
    }

    pub fn prepare_to_start(&mut self) -> Result<(), Error> {
        self.install_software()?;
        self.install_system_plugin()?;
        self.install_software_lib()?;
        self.set_server_address()?;
        self.find_new_free_plugin_listener();
        // muss hier sonst holg set_server_address && find_new_free_plugin_listener sich sein eigenen port
        self.set_status(ServiceStatus::Prepare);
        Ok(())
    }

    pub fn get_service_url(&self) -> Url {
        Url::new(
            UrlSchema::Http,
            &self.get_plugin_listener(),
            "cloud/service",
        )
        .join(&self.get_name())
    }

    pub async fn connect_to_network(&self, cloud: Arc<RwLock<Cloud>>) -> Result<(), Error> {
        if self.is_proxy() {
            // TODO: Send New Started Proxy Service To Cluster
            return Ok(());
        }

        let services = { cloud.read().await.get_local().get_started_proxy_services() };

        for service_proxy in services {
            let url = service_proxy.get_service_url().join("add_server");
            let body = match Utils::convert_to_json(&RegisterServerData {
                register_server: ServiceInfoResponse::new(self),
            }) {
                Some(body) => body,
                None => {
                    log_warning!("Service {} can't Serialize to ServiceInfo", self.get_name());
                    continue;
                }
            };

            match url.post(&body, Duration::from_secs(3)).await {
                Ok(_) => log_info!(
                "Service {} successfully connected to Proxy [{}]",
                self.get_name(),
                    service_proxy.get_name()
            ),
                Err(e) => log_warning!(
                "Service | {} | can't send request connect to Network \n Error: {}",
                self.get_name(),
                e.to_string()
            ),
            }
        }

        // TODO: Send New Started Service To Cluster
        Ok(())
    }


    pub async fn disconnect_from_network(&self, cloud: Arc<RwLock<Cloud>>) -> Result<(), Error> {
        if self.is_proxy() {
            return Ok(());
            // TODO: Send New Stopped Proxy Service To Cluster
        }

        let services = { cloud.read().await.get_local().get_started_proxy_services() };

        for service_proxy in services {
            let url = service_proxy
                .get_service_url()
                .join(format!("remove_server?name={}", self.get_name()).as_str());
            match url.post(&json!({}), Duration::from_secs(3)).await {
                Ok(_) => log_info!(
                    "Service {} successfully disconnected from Proxy [{}]",
                    self.get_name(),
                    service_proxy.get_name()
                ),
                Err(e) => log_warning!(
                    "Service | {} | can't send request disconnect from Network \n Error: {}",
                    self.get_name(),
                    e.to_string()
                ),
            }
        }
        // TODO: Send New Stopped Service To Cluster
        Ok(())
    }

    pub fn start(mut self: Service) -> Result<Service, Error> {
        self.prepare_to_start()?;

        let server_file_path = match self.get_path_with_server_file().to_str() {
            Some(server_file_path) => server_file_path.to_string(),
            None => {
                return Err(Error::new(
                    ErrorKind::Other,
                    "Can not server file path to string change",
                ));
            }
        };

        let server_path = match self.get_path().to_str() {
            Some(server_file_path) => server_file_path.to_string(),
            None => {
                return Err(Error::new(
                    ErrorKind::Other,
                    "Can not server path to string change",
                ));
            }
        };

        let software_name = self.get_software_name();
        let mut placeholders = HashMap::new();
        let stdout_file = File::create(self.get_path_stdout_file())?;
        let stderr_file = File::create(self.get_path_stderr_file())?;

        placeholders.insert("ip", self.get_server_address().get_ip().to_string());
        placeholders.insert("port", self.get_server_address().get_port().to_string());
        placeholders.insert("max_ram", software_name.get_max_ram().to_string());
        placeholders.insert("server_file", server_file_path);

        let process_args = Utils::replace_placeholders(
            software_name.get_environment().get_process_args(),
            &placeholders,
        );

        let child = Command::new(software_name.get_environment().get_command())
            .args(&process_args)
            .current_dir(server_path)
            .stdout(Stdio::from(stdout_file))
            .stderr(Stdio::from(stderr_file))
            .stdin(Stdio::piped())
            .spawn()?;

        self.set_process(Some(child));
        Ok(self)
    }

    pub fn get_from_name(name: &String) -> Option<Service> {
        let mut path = CloudConfig::get()
            .get_cloud_path()
            .get_service_folder()
            .get_temp_folder_path()
            .join(&name);
        Service::get_from_path(&mut path)
    }

    pub fn install_software(&self) -> Result<(), Error> {
        let target_path = self
            .get_path()
            .join(&self.get_task().get_software().get_server_file_name());
        let software_path = self.get_task().get_software().get_software_file_path();

        fs::copy(&software_path, &target_path)?;
        Ok(())
    }

    pub fn install_system_plugin(&self) -> Result<(), Error> {
        let software = self.get_software_name();
        let system_plugin_path = self.get_task().get_software().get_system_plugin_path();
        let mut target_path = self
            .get_path()
            .join(&software.get_system_plugin().get_path());

        if !target_path.exists() {
            fs::create_dir_all(&target_path)?;
        }

        target_path.push(self.get_task().get_software().get_system_plugin_name());

        if !system_plugin_path.exists() {
            return Err(Error::new(
                ErrorKind::Other,
                format!(
                    "System plugin Esitiert nicht wie im Path angegeben {}",
                    system_plugin_path.to_str().unwrap()
                ),
            ));
        }

        match fs::copy(system_plugin_path, target_path) {
            Ok(_) => {
                log_info!("Successfully install the System Plugin");
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn install_software_lib(&self) -> Result<(), Error> {
        let software_lib_path = CloudConfig::get()
            .get_cloud_path()
            .get_system_folder()
            .get_software_lib_folder_path()
            .join(self.get_task().get_software().get_software_type())
            .join(self.get_task().get_software().get_name());

        match Directory::copy_folder_contents(&software_lib_path, &self.get_path()) {
            Ok(()) => Ok(()),
            Err(e) => Err(Error::new(ErrorKind::Other, e.to_string())),
        }
    }

    pub fn is_proxy(&self) -> bool {
        self.get_software_name().get_server_type().is_proxy()
    }

    pub fn is_backend_server(&self) -> bool {
        self.get_software_name()
            .get_server_type()
            .is_backend_server()
    }
}

fn find_port(ports: Vec<u32>, mut port: u32, server_host: &String) -> u32 {
    while ports.contains(&port) || !Address::is_port_available(&Address::new(&server_host, &port)) {
        port = Address::find_next_port(&mut Address::new(&server_host, &(port + 1)));
    }
    port
}
