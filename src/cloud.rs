use colored::{ColoredString, Colorize};
use database_manager::DatabaseManager;
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use std::{env, fs};
use tokio::sync::RwLock;

use crate::api::internal::APIInternal;
use crate::config::{CloudConfig, SoftwareConfig, SoftwareConfigRef};
use crate::database::table::Tables;
use crate::manager::{GroupManagerRef, Manager, NodeManager, PlayerManager, TaskManagerRef};
use crate::node::scheduler::Scheduler;
use crate::terminal::cmd::Cmd;
use crate::utils::error::*;
use crate::utils::log::logger::Logger;
use crate::log_info;

#[cfg(feature = "rest-api")]
use crate::api::external::restapi_main::ApiMain;

pub struct Cloud {
    config: Arc<CloudConfig>,
    software_config: SoftwareConfigRef,
    db: Arc<DatabaseManager>,
    scheduler: Arc<Scheduler>,
    task_manager: TaskManagerRef,
    node_manager: Arc<NodeManager>,
    player_manager: Arc<PlayerManager>,
    group_manager: GroupManagerRef,
}

impl Cloud {
    pub async fn new(cloud_config: CloudConfig, url: String) -> CloudResult<Self> {
        let config = Arc::new(cloud_config);
        let software_config = SoftwareConfigRef::new(SoftwareConfig::check_and_get(config.clone(), &url).await.expect("Checking Software failed"));
        let mut db = DatabaseManager::new(config.get_db_config())?;
        db.connect().await?;
        let db = Arc::new(db);
        Tables::check_tables(db.as_ref()).await?;
        log_info!("Database check successfully");

        let (pm, tm, nm, gm) =
            Manager::create_all(db.clone(), config.clone(), software_config.clone()).await?;
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
            group_manager: gm,
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

        let cloud = Arc::new(RwLock::new(
            Cloud::new(cloud_config, url).await.expect("Cant Create Cloud"),
        ));

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
            let _scheduler = cloud_clone.scheduler.clone();

            tokio::spawn(async move {
                //scheduler.run().await;
            });
            log_info!(3, "Scheduler started!");
        }

        cmd.start().await;
        Ok(())
    }

    pub async fn disable(&mut self) {
        self.node_manager
            .stop_all_local_services("Cloud Disable")
            .await;
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

        // create software folder
        fs::create_dir_all(config_path.get_system_folder().get_software_config_path())?;

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

}
