use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time;

use crate::config::cloud_config::CloudConfig;
use crate::config::software_config::SoftwareConfig;
use crate::database::manager::DatabaseManager;
use crate::{log_error, log_info};
use crate::manager::service_manager::ServiceManager;
use crate::manager::task_manager::TaskManager;

pub struct Scheduler {
    db: Arc<DatabaseManager>,
    config: Arc<CloudConfig>,
    software_config: Arc<RwLock<SoftwareConfig>>,
    service_manager: Arc<RwLock<ServiceManager>>,
    task_manager: Arc<RwLock<TaskManager>>,
}


impl Scheduler {
    pub fn new(db: Arc<DatabaseManager>,
               config: Arc<CloudConfig>,
               software_config: Arc<RwLock<SoftwareConfig>>,
               service_manager: Arc<RwLock<ServiceManager>>,
               task_manager: Arc<RwLock<TaskManager>>
    ) -> Scheduler {
        Scheduler {
            db,
            config,
            software_config,
            service_manager,
            task_manager,
        }
    }

    pub async fn run(&self) {
        let mut interval = time::interval(Duration::from_secs(10));

        loop {
            interval.tick().await;
            log_info!(9, "Scheduler Checking...");
            self.check_service().await;
        }

    }

    pub async fn check_service(&self) {
        let tasks = {
            self.task_manager.read().await.get_all_task()
        };

        for task in tasks {
            if !task.is_startup_local(&self.config) {
                continue;
            }

            let task_name = task.get_name();

            // 🔒 nur kurz lesen
            let services = {
                let sm = self.service_manager.read().await;
                sm.get_all_from_task(&task_name)
            };

            log_info!(
            "(Local) Task: {} Services: | Start: {} | Stop: {} | Failed: {}",
            task_name,
            services.iter().filter(|s| s.is_start()).count(),
            services.iter().filter(|s| s.is_stop()).count(),
            services.iter().filter(|s| s.is_failed()).count()
        );

            let missing = task.get_min_service_count()
                - services.iter().filter(|s| s.is_start()).count() as u64;

            for _ in 0..missing {
                log_info!("---------------------------------------------------------------");
                log_info!("Service would be created from task: [{}]", task_name);

                let service = {
                    let mut sm = self.service_manager.read().await;
                    match sm.create_service(&task) {
                        Ok(sm) => sm,
                        Err(e) => {
                            log_error!(1, "Cant Create Service Error: {}", e);
                            continue;
                        },
                    }
                };

                match service.start_async().await {
                    Ok(started) => {
                        log_info!(
                        2,
                        "Server [{}] successfully start :=)",
                        started.get_service().get_name()
                    );
                        let mut sm = self.service_manager.write().await;
                        sm.set_service(started);
                    }
                    Err(e) => {
                        log_error!(
                        1,
                        "Service CANT Start Task: [{}]\nError: {}",
                        task_name,
                        e
                    );
                    }
                }
            }
        }
    }
}
