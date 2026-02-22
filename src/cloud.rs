use bx::network::url::Url;
use colored::{ColoredString, Colorize};
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use std::{env, fs};
use database_manager::DatabaseManager;
use tokio::sync::RwLock;

use crate::terminal::cmd::Cmd;
use crate::utils::log::logger::Logger;
use crate::{log_error, log_info, log_warning};
use crate::utils::error::*;
use crate::node::scheduler::Scheduler;
use crate::database::table::Tables;
use crate::manager::{Manager, NodeManager, PlayerManager, TaskManager};
use crate::config::{CloudConfig, SoftwareConfig, SoftwareConfigRef};
use crate::api::internal::APIInternal;


#[cfg(feature = "rest-api")]
use crate::api::external::restapi_main::ApiMain;


pub struct Cloud {
    config: Arc<CloudConfig>,
    software_config: SoftwareConfigRef,
    db: Arc<DatabaseManager>,
    scheduler: Arc<Scheduler>,
    task_manager: Arc<TaskManager>,
    node_manager: Arc<NodeManager>,
    player_manager: Arc<PlayerManager>,
}

impl Cloud {
    pub async fn new(cloud_config: CloudConfig) -> CloudResult<Self> {
        let config = Arc::new(cloud_config);
        let software_config = SoftwareConfigRef::new(config.clone());
        let mut db = DatabaseManager::new(config.get_db_config())?;
        db.connect().await?;
        let db = Arc::new(db);
        Tables::check_tables(db.as_ref()).await?;
        log_info!("Database check successfully");

        let (pm, tm, nm) = Manager::create_all(db.clone(), config.clone(), software_config.clone()).await?;
        let scheduler = Arc::new(Scheduler::new(
            db.clone(),
            config.clone(),
            software_config.clone(),
            nm.clone(),
            tm.clone(),
        ));

        Ok(Self {
            config,
            software_config,
            db,
            scheduler,
            node_manager: nm,
            task_manager: tm,
            player_manager: pm,
        })
    }

    pub fn get_config(&self) -> &CloudConfig {
        &self.config
    }
    pub fn get_db(&self) -> &Arc<DatabaseManager> {
        &self.db
    }
    pub fn get_node_manager(&self) -> Arc<NodeManager> {
        self.node_manager.clone()
    }
    pub fn get_scheduler(&self) -> &Arc<Scheduler> {
        &self.scheduler
    }
    pub fn get_player_manager(&self) -> Arc<PlayerManager> {
        self.player_manager.clone()
    }

    pub async fn enable(version: &str) -> CloudResult<()> {
        // download link
        let url = format!(
            "http://download.codergames.de/minecloud/version/{}/",
            version
        );

        // print the logo
        Cloud::print_icon();

        //check the cloud config.json
        let cloud_config = CloudConfig::check_and_get(&url).await;
        Logger::init_log_level(cloud_config.get_log_level());
        
        // check folder
        Cloud::check_folder(&cloud_config).expect("Checking Folder failed");

        // check software config file
        SoftwareConfig::check(&url).await;

        // check the software files
        Cloud::check_software(&cloud_config)
            .await
            .expect("Checking Software failed");

        let cloud = Arc::new(RwLock::new(Cloud::new(cloud_config).await.expect("Cant Create Cloud")));

        // Internal API
        APIInternal::start(cloud.clone()).await?;

        #[cfg(feature = "rest-api")]
        {
            let cloud_clone = cloud.clone();
            std::thread::spawn(move || {
                //let _ = ApiMain::start(cloud_clone);
            });
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
        let cmd = Cmd::new(
            &ColoredString::from(CloudConfig::get().get_prefix().as_str()).cyan(),
            cloud.clone(),
        );

        {
            let cloud_clone = cloud.read().await;
            let scheduler = cloud_clone.scheduler.clone();

            tokio::spawn(async move {
                //scheduler.run().await;
            });
            log_info!(3, "Scheduler started!");
        }

        cmd.start().await;
        Ok(())
    }

    pub async fn disable(&mut self) {
        self.node_manager.stop_all_local_services("Cloud Disable").await;
        log_info!("Cloud shutdown");
        log_info!("bye bye");
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

    pub fn check_folder(cloud_config: &CloudConfig) -> Result<(), Box<dyn Error>> {
        let config_path = cloud_config.get_cloud_path();

        // create task folder
        fs::create_dir_all(config_path.get_task_folder_path())?;

        // create template folder
        fs::create_dir_all(config_path.get_template_folder_path())?;

        // create group folder
        fs::create_dir_all(config_path.get_group_folder_path())?;

        // create service temp folder
        fs::create_dir_all(config_path.get_service_folder().get_temp_folder_path())?;

        // create service static folder
        fs::create_dir_all(config_path.get_service_folder().get_static_folder_path())?;

        // create system_plugins_folder
        fs::create_dir_all(
            config_path
                .get_system_folder()
                .get_system_plugins_folder_path(),
        )?;

        // create software_files_folder
        fs::create_dir_all(
            config_path
                .get_system_folder()
                .get_software_files_folder_path(),
        )?;

        // create software_lib_folder
        fs::create_dir_all(
            config_path
                .get_system_folder()
                .get_software_lib_folder_path(),
        )?;
        log_info!(2, "All Folders are safe :=)");
        Ok(())
    }

    // check software && system plugins
    pub async fn check_software(cloud_config: &CloudConfig) -> Result<(), Box<dyn Error>> {
        // Todo: refactoren 'check_software()'
        let software_types = SoftwareConfig::get().get_software_types();
        let cloud_config_system = cloud_config.get_cloud_path().get_system_folder();

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


