use std::sync::Arc;
use std::time::Duration;
use database_manager::DatabaseManager;
use tokio::sync::RwLock;
use tokio::time;
use tokio::time::Instant;

use crate::{log_error, log_info};
use crate::config::{CloudConfig, SoftwareConfigRef};
use crate::database::table::TablePlayerSessions;
use crate::manager::{NodeManager, TaskManager};
use crate::types::Task;
use crate::utils::error::CloudResult;

pub struct Scheduler {
    db: Arc<DatabaseManager>,
    config: Arc<CloudConfig>,
    software_config: SoftwareConfigRef,
    node_manager: Arc<NodeManager>,
    task_manager: Arc<TaskManager>,
    last_scale_action: Arc<RwLock<Option<Instant>>>,
}


impl Scheduler {
    pub fn new(db: Arc<DatabaseManager>,
               config: Arc<CloudConfig>,
               software_config: SoftwareConfigRef,
               node_manager: Arc<NodeManager>,
               task_manager: Arc<TaskManager>
    ) -> Scheduler {
        Scheduler {
            db,
            config,
            software_config,
            node_manager,
            task_manager,
            last_scale_action: Arc::new(RwLock::new(None)),
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
            // 🔒 nur kurz lesen, um alle Tasks zu kopieren
            self.task_manager.read().await.get_all_task()
        };
        todo!();
        for task in tasks {
            if !task.is_startup_local(&self.config) {
                continue;
            }

            let task_name = task.get_name();

            let services = {
                let sm = self.service_manager.read().await;
                sm.get_all_from_task(&task_name)
            };

            let start_count = services.iter().filter(|s| s.is_start()).count();
            let stop_count = services.iter().filter(|s| s.is_stop()).count();
            let failed_count = services.iter().filter(|s| s.is_failed()).count();

            log_info!(
            "(Local) Task: {} Services: | Start: {} | Stop: {} | Failed: {}",
            task_name,
            start_count,
            stop_count,
            failed_count
        );

            let missing = task.get_min_service_count().saturating_sub(start_count as u64);
            for _ in 0..missing {
                log_info!("---------------------------------------------------------------");
                log_info!("Service would be created from Task: [{}]", task_name);

                let service_result = {
                    let sm = self.service_manager.read().await;
                    sm.get_or_create_service_process(&task).await
                };

                let service_process = match service_result {
                    Ok(proc) => proc,
                    Err(e) => {
                        log_error!(
                        1,
                        "Service CANT create/get for Task: [{}]\nError: {}",
                        task_name,
                        e
                    );
                        continue;
                    }
                };

                let started_service = match self.service_manager.read().await.start_service(service_process).await {
                    Ok(s) => {
                        log_info!(2, "Server [{}] successfully started", s.get_service().get_name());
                        s
                    }
                    Err(e) => {
                        log_error!(1, "Service CANT start for Task: [{}]\nError: {}", task_name, e);
                        continue;
                    }
                };

                {
                    let mut sm = self.service_manager.write().await;
                    sm.set_service(started_service);
                }
            }
        }
    }

    pub async fn check_player_scaling_all(&self) -> CloudResult<()> {

        let tasks = {
            self.task_manager.read().await.get_all_task()
        };

        for task in tasks {
            if !task.is_startup_local(&self.config) {
                continue;
            }

            self.check_player_scaling_by_task(&task).await;
        }
        Ok(())
    }

    async fn check_player_scaling_by_task(&self, task: &Task) {
        {
            let last = self.last_scale_action.read().await;
            if let Some(last_action) = *last {
                if last_action.elapsed() < Duration::from_secs(5) {
                    return;
                }
            }
        }

        let running_services = {
            let sm = self.service_manager.read().await;
            sm.get_online_all_from_task(&task.get_name())
        };
        todo!();

        if running_services.is_empty() {
            return;
        }

        let total_players = match TablePlayerSessions::count_players_from_task(self.get_db(), &task.get_name()).await {
            Ok(count) => count,
            Err(e) => {
                log_error!("[Scheduler] {}", e.to_string());
                return;
            }
        };

        let max_possible_players =
            running_services.len() as u32 * task.get_max_players();

        if max_possible_players == 0 {
            return;
        }

        let usage_percent = (total_players * 100) / max_possible_players as u64;

        log_info!(3,
            "[Scaling] Task: {} | Players: {} | Usage: {}%",
            task.get_name(),
            total_players,
            usage_percent
        );

        if usage_percent
            >= task.get_percent_of_players_for_a_new_service_by_instance() as u64
        {
            self.scaling_up(task).await;
        } else {
            //self.scaling_down(1, task, usage_percent).await;
        }
    }

    async fn scaling_up(&self, task: &Task) {
        let running_services = {
            let sm = self.service_manager.read().await;
            sm.get_online_all_from_task(&task.get_name())
        };
        todo!();
        if running_services.len() as u64
            >= task.get_max_service_count() as u64
        {
            return;
        }

        log_info!(
        "[Scaling] Starting new service for {}",
        task.get_name()
    );

        // get_or_create + start außerhalb von write-lock
        let started = {
            let sm = self.service_manager.read().await;

            match sm.get_or_create_service_process(task).await {
                Ok(proc) => {
                    match sm.start_service(proc).await {
                        Ok(s) => s,
                        Err(e) => {
                            log_error!("Scaling start failed: {}", e);
                            return;
                        }
                    }
                }
                Err(e) => {
                    log_error!("Scaling create failed: {}", e);
                    return;
                }
            }
        };

        {
            let mut sm = self.service_manager.write().await;
            sm.set_service(started);
        }

        *self.last_scale_action.write().await =
            Some(Instant::now());
    }


    async fn scaling_down(&self, running_services: u32, task: &Task, usage_percent: u64) {
        if usage_percent
            < task.get_percent_of_players_to_check_should_auto_stop_the_service() as u64
        {
            /*if running > task.get_min_non_full_service() {

                if let Some(service) = running_services
                    .iter()
                    .find(|s|
                        s.get_online_players() == 0 &&
                            s.unused_for_seconds()
                                >= task.auto_stop_time_by_unused_service_in_seconds
                    )
                {
                    sm.stop_service(service.get_id(), "Auto scaling down").await?;
                }
            }*/
        }

    }


    fn get_db(&self) -> &DatabaseManager {
        self.db.as_ref()
    }

}
