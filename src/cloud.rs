use std::error::Error;
use bx::network::url::Url;
use colored::{ColoredString, Colorize};
use std::{env, fs};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::core::services_all::AllServices;
use crate::core::services_local::LocalServices;
use crate::core::services_network::NetworkServices;
use crate::node_api::node_main::NodeServer;
use crate::sys_config::cloud_config::CloudConfig;
use crate::sys_config::software_config::SoftwareConfig;
use crate::terminal::cmd::Cmd;
use crate::utils::logger::Logger;
use crate::{log_error, log_info, log_warning};


#[cfg(feature = "rest-api")]
use crate::rest_api::restapi_main::ApiMain;
use crate::utils::service_status::ServiceStatus;

pub struct Cloud {
    services: AllServices,
}

impl Cloud {
    pub fn new() -> Self {
        // wenns beim runterfahren geknallt hat un in den services datein noch start oder Prepare steht
        Cloud::set_stop_status_service();
        let local = LocalServices::new();
        let network = NetworkServices::new();
        let all = AllServices::new(local, network); // initialisieren
        Self { services: all }
    }

    pub fn get_all(&self) -> &AllServices {
        &self.services
    }

    pub fn get_all_mut(&mut self) -> &mut AllServices {
        &mut self.services
    }

    pub fn get_local(&self) -> &LocalServices {
        &self.services.get_local()
    }

    pub fn get_local_mut(&mut self) -> &mut LocalServices {
        self.services.get_local_mut()
    }

    pub fn get_network(&self) -> &NetworkServices {
        &self.services.get_network()
    }

    pub fn get_network_mut(&mut self) -> &mut NetworkServices {
        self.services.get_network_mut()
    }

    pub async fn enable(cloud: Arc<RwLock<Cloud>>, version: &str) {
        // download link
        let url = format!(
            "http://download.codergames.de/minecloud/version/{}/",
            version
        );

        // print the logo
        Cloud::print_icon();

        //check the cloud config.json
        CloudConfig::check(&url).await;

        // check folder
        Cloud::check_folder().expect("Checking Folder failed");

        // check software config file
        SoftwareConfig::check(&url).await;

        // check the software files
        Cloud::check_software().await.expect("Checking Software failed");

        // NodeServer
        {
            let cloud_clone = cloud.clone();
            std::thread::spawn(move || {
                let _ = NodeServer::start(cloud_clone);
            });
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
        #[cfg(feature = "rest-api")]
        {
            let cloud_clone = cloud.clone();
            std::thread::spawn(move || {
                let _ = ApiMain::start(cloud_clone);
            });
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
        let cmd = Cmd::new(
            &ColoredString::from(CloudConfig::get().get_prefix().as_str()).cyan(),
            cloud.clone(),
        );
        cmd.start().await;
    }

    pub async fn disable(&mut self) {
        self.get_local_mut().stop_all("Cloud Disable").await;
        log_info!("Cloud shutdown");
        log_info!("Bye Bye");
        std::process::exit(0)
    }

    pub fn get_working_path() -> PathBuf {
        let mut custom: Option<PathBuf> = None;

        for arg in env::args().skip(1) {
            if let Some(rest) = arg.strip_prefix("-working-path=") {
                custom = Some(PathBuf::from(rest));
            }
        }

        // Wenn ein custom working path angegeben wurde
        if let Some(path) = custom {
            if path.exists() && path.is_dir() {
                return path;
            } else {
                eprintln!("Ungültiger Pfad bei -working-path: {}", path.display());
            }
        }

        // Fallback: EXE-Verzeichnis
        match env::current_exe() {
            Ok(mut exe) => {
                exe.pop();
                exe
            }
            Err(e) => {
                eprintln!("Error getting exe path: {}", e);
                panic!("Fatal error")
            }
        }
    }


    pub fn print_icon() {
        println!(" ");
        println!(
            "_____{}__________________________________________________________{}__{}________________________________________{}_____",
            r"/\\\\\\\\\\\\".red(),
            r"/\\\\\\\\\".cyan(),
            r"/\\\\\\".cyan(),
            r"/\\\".cyan()
        );
        println!(
            "___{}________________________________________________________{}__{}_______________________________________{}_____",
            r"/\\\//////////".red(),
            r"/\\\////////".cyan(),
            r"\////\\\".cyan(),
            r"\/\\\".cyan()
        );
        println!(
            "__{}_________________________________________________________________{}______________{}_______________________________________{}_____",
            r"/\\\".red(),
            r"/\\\/".cyan(),
            r"\/\\\".cyan(),
            r"\/\\\".cyan()
        );
        println!(
            "_{}____{}__{}_______{}__{}_______{}___{}________________{}________{}_____{}____{}________{}_____",
            r"\/\\\".red(),
            r"/\\\\\\\".red(),
            r"/\\\\\\\\\".red(),
            r"/\\\\\".red(),
            r"/\\\\\".red(),
            r"/\\\\\\\\".red(),
            r"/\\\".cyan(),
            r"\/\\\".cyan(),
            r"/\\\\\".cyan(),
            r"/\\\".cyan(),
            r"/\\\".cyan(),
            r"\/\\\".cyan()
        );
        println!(
            "_{}___{}_{}____{}___{}_{}________________{}______{}__{}___{}___{}_____",
            r"\/\\\".red(),
            r"\/////\\\".red(),
            r"\////////\\\".red(),
            r"/\\\///\\\\\///\\\".red(),
            r"/\\\/////\\\".red(),
            r"\/\\\".cyan(),
            r"\/\\\".cyan(),
            r"/\\\///\\\".cyan(),
            r"\/\\\".cyan(),
            r"\/\\\".cyan(),
            r"/\\\\\\\\\".cyan()
        );
        println!(
            "__{}_______{}___{}__{}_{}__{}__{}__{}_______________{}_____{}__{}_{}___{}__{}____",
            r"\/\\\".red(),
            r"\/\\\".red(),
            r"/\\\\\\\\\\".red(),
            r"\/\\\".red(),
            r"\//\\\".red(),
            r"\/\\\".red(),
            r"/\\\\\\\\\\\".red(),
            r"\//\\\".cyan(),
            r"\/\\\".cyan(),
            r"/\\\".cyan(),
            r"\//\\\".cyan(),
            r"\/\\\".cyan(),
            r"\/\\\".cyan(),
            r"/\\\////\\\".cyan()
        );
        println!(
            "___{}_______{}__{}__{}__{}__{}_{}____{}_____________{}____{}__{}__{}___{}_{}__{}___",
            r"\/\\\".red(),
            r"\/\\\".red(),
            r"/\\\/////\\\".red(),
            r"\/\\\".red(),
            r"\/\\\".red(),
            r"\/\\\".red(),
            r"\//\\///////".red(),
            r"\///\\\".cyan(),
            r"\/\\\".cyan(),
            r"\//\\\".cyan(),
            r"/\\\".cyan(),
            r"\/\\\".cyan(),
            r"\/\\\".cyan(),
            r"\/\\\".cyan(),
            r"\/\\\".cyan()
        );
        println!(
            "____{}__{}_{}__{}__{}__{}____{}__{}__{}___{}__{}_",
            r"\//\\\\\\\\\\\\/".red(),
            r"\//\\\\\\\\/\\".red(),
            r"\/\\\".red(),
            r"\/\\\".red(),
            r"\/\\\".red(),
            r"\//\\\\\\\\\\".red(),
            r"\////\\\\\\\\\".cyan(),
            r"/\\\\\\\\\".cyan(),
            r"\///\\\\\/".cyan(),
            r"\//\\\\\\\\\".cyan(),
            r"\//\\\\\\\/\\".cyan()
        );
        println!(
            "_____{}_____{}__{}___{}___{}____{}________{}__{}_____{}______{}____{}__",
            r"\////////////".red(),
            r"\////////\//".red(),
            r"\///".red(),
            r"\///".red(),
            r"\///".red(),
            r"\//////////".red(),
            r"\/////////".cyan(),
            r"\/////////".cyan(),
            r"\/////".cyan(),
            r"\/////////".cyan(),
            r"\///////\//".cyan()
        );
        println!(" ");
    }

    pub fn set_stop_status_service() {
        // nur die loken services holen nicht die aus dem netzwerk
        // wichtig Service::get_all() lassen denn hier muss expliziet in die datein gegucklt werden
        for mut service in LocalServices::get_all_from_file() {
            service.set_status(ServiceStatus::Stop);
            service.save_to_file();
        }
    }

    pub fn check_folder() -> Result<(), Box<dyn Error>> {
        let config_path = CloudConfig::get().get_cloud_path();

        // create task folder
        fs::create_dir_all(&config_path.get_task_folder_path())?;

        // create template folder
        fs::create_dir_all(&config_path.get_template_folder_path())?;

        // create service temp folder
        fs::create_dir_all(&config_path.get_service_folder().get_temp_folder_path())?;

        // create service static folder
        fs::create_dir_all(&config_path.get_service_folder().get_static_folder_path())?;

        // create system_plugins_folder
        fs::create_dir_all(
            &config_path
                .get_system_folder()
                .get_system_plugins_folder_path(),
        )?;

        // create software_files_folder
        fs::create_dir_all(
            &config_path
                .get_system_folder()
                .get_software_files_folder_path(),
        )?;

        // create software_lib_folder
        fs::create_dir_all(
            &config_path
                .get_system_folder()
                .get_software_lib_folder_path(),
        )?;
        log_info!("All Folders are safe :=)");
        Ok(())
    }

    // check software && system plugins
    pub async fn check_software() -> Result<(), Box<dyn Error>> {
        // Todo: refactoren 'check_software()'
        let software_types = SoftwareConfig::get().get_software_types();
        let cloud_config_system = CloudConfig::get().get_cloud_path().get_system_folder();

        // iter to software types
        for (software_type_name, software_type) in software_types {
            // create var paths
            let software_path = cloud_config_system
                .get_software_files_folder_path()
                .join(&software_type_name);
            let system_plugins_path = cloud_config_system
                .get_system_plugins_folder_path()
                .join(&software_type_name);
            let software_lib_path = cloud_config_system
                .get_software_lib_folder_path()
                .join(&software_type_name);

            // create the software types folder
            fs::create_dir_all(&software_path)?;
            fs::create_dir_all(&system_plugins_path)?;
            fs::create_dir_all(&software_lib_path)?;

            // iter to software names
            for software in software_type.get_software_names() {
                let software_file_url = software.get_software_file().get_url();
                let system_plugins_path = match Url::extract_extension_from_url(
                    &software.get_system_plugin().get_download(),
                ) {
                    Some(ext) => system_plugins_path.join(format!(
                        "MineCloud-{}.{}",
                        software.get_name(),
                        ext
                    )),
                    None => system_plugins_path.join(software.get_name()),
                };

                let software_path = match Url::extract_extension_from_url(&software_file_url) {
                    Some(ext) => software_path.join(format!("{}.{}", software.get_name(), ext)),
                    None => software_path.join(software.get_name()),
                };

                // download software when software file does not exist
                if !software_path.exists() {
                    log_info!("Download Software {}", software.get_name());
                    match Url::download_file(&*software_file_url, &software_path).await {
                        Ok(_) => {
                            log_info!(
                                "Successfully download the Software from url {}",
                                software_file_url
                            );
                        }
                        Err(e) => {
                            log_error!("{}", e.to_string());
                            return Err(e);
                        }
                    }
                }

                // download system plugin when plugin file does not exist
                if !software.get_system_plugin().is_local() && !system_plugins_path.exists() {
                    log_info!(
                        "Download Software System Plugin {} Plugin",
                        software.get_name()
                    );
                    match Url::download_file(
                        software.get_system_plugin().get_download().as_str(),
                        &system_plugins_path,
                    )
                    .await
                    {
                        Ok(_) => {
                            log_info!(
                                "Successfully download the Software System Plugin from url {}",
                                software.get_system_plugin().get_download()
                            );
                        }
                        Err(e) => {
                            log_error!("{}", e.to_string());
                            return Err(e);
                        }
                    }
                }
                let software_lib_list = software.get_software_lib();

                if software_lib_list.is_empty() {
                    continue;
                }

                for (url_str, software_lib_path) in software_lib_list {
                    if !software_lib_path.exists() || software.get_software_file().is_auto_update()
                    {
                        let mut path = software_lib_path.clone();
                        path.pop();
                        fs::create_dir_all(&path)?;

                        match Url::download_file(&url_str, &software_lib_path).await {
                            Ok(_) => log_info!(
                                "Successfuly donwload software lib from {} to {:?}",
                                url_str,
                                software_lib_path
                            ),
                            Err(e) => log_warning!(
                                "Software Lib cant download {} \n {}",
                                url_str,
                                e.to_string()
                            ),
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

