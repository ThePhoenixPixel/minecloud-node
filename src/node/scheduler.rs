use database_manager::DatabaseManager;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time;
use tokio::time::Instant;

use crate::config::{CloudConfig, SoftwareConfigRef};
use crate::manager::{NodeManager, TaskManagerRef};
use crate::utils::error::CloudResult;
use crate::{log_error, log_info};

pub struct Scheduler {
    db: Arc<DatabaseManager>,
    config: Arc<CloudConfig>,
    software_config: SoftwareConfigRef,
    node_manager: Arc<NodeManager>,
    task_manager: TaskManagerRef,
    last_scale_action: Arc<RwLock<Option<Instant>>>,
}

impl Scheduler {
    pub fn new(
        db: Arc<DatabaseManager>,
        config: Arc<CloudConfig>,
        software_config: SoftwareConfigRef,
        node_manager: Arc<NodeManager>,
        task_manager: TaskManagerRef,
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

    pub async fn run(&mut self) {
        let mut interval = time::interval(Duration::from_secs(10));

        loop {
            interval.tick().await;
            log_info!(9, "Scheduler Checking...");
            self.check_service().await;
        }
    }

    pub async fn check_service(&self) {
        let tasks = {
            let tm = self.task_manager.read().await;
            tm.get_all_tasks()
        };

        for task_ref in tasks {
            let task = task_ref.read().await;

            if !self.node_manager.is_responsible_for_task(&task).await {
                continue;
            }

            let task_name = task.get_name();
            let services = self
                .node_manager
                .get_all_services_from_task(&task_name)
                .await;
            let start_count = services.iter().filter(|s| s.is_start()).count() as u64;
            let stop_count = services.iter().filter(|s| s.is_stop()).count() as u64;
            let failed_count = services.iter().filter(|s| s.is_failed()).count() as u64;

            log_info!(
                "Task: {} Services: | Start: {} | Stop: {} | Failed: {}",
                task_name,
                start_count,
                stop_count,
                failed_count
            );

            let missing = task.get_min_service_count().saturating_sub(start_count);
            for _ in 0..missing {
                log_info!("---------------------------------------------------------------");
                log_info!("Service would be created from Task: [{}]", task_name);

                match self.node_manager.start_service_from_task(&task).await {
                    Ok(_) => {
                        log_info!(2, "Server successfully started for Task [{}]", task_name);
                    }
                    Err(e) => {
                        log_error!(
                            1,
                            "Service CANT start for Task: [{}]\nError: {}",
                            task_name,
                            e
                        );
                    }
                }
            }
        }
    }

    /*pub async fn check_player_scaling_all(&self) -> CloudResult<()> {
        let tasks = {
            let tm = self.task_manager.read().await;
            tm.get_all_task()
        };

        for task_ref in tasks {
            let task = task_ref.read().await;

            if !self.node_manager.is_responsible_for_task(&task).await {
                continue;
            }

            // self.check_player_scaling_by_task(&task).await;
        }
        Ok(())
    }*/
}
/*


async fn check_player_scaling_by_task(&self, task: &Task) {
    {
        let last = self.last_scale_action.read().await;
        if let Some(last_action) = *last {
            if last_action.elapsed() < Duration::from_secs(5) {
                return;
            }
        }
    }

    let running_services = self.node_manager.get_online_all_from_task(&task.get_name()).await;

    if running_services.is_empty() {
        return;
    }

    let total_players = match TablePlayerSessions::count_players_from_task(
        self.get_db(),
        &task.get_name(),
    )
        .await
    {
        Ok(count) => count,
        Err(e) => {
            log_error!("[Scheduler] {}", e.to_string());
            return;
        }
    };

    let max_possible_players = running_services.len() as u32 * task.get_max_players();

    if max_possible_players == 0 {
        return;
    }

    let usage_percent = (total_players * 100) / max_possible_players as u64;

    log_info!(
        3,
        "[Scaling] Task: {} | Players: {} | Usage: {}%",
        task.get_name(),
        total_players,
        usage_percent
    );

    if usage_percent >= task.get_percent_of_players_for_a_new_service_by_instance() as u64 {
        self.scaling_up(task).await;
    } else {
        self.scaling_down(&running_services, task, usage_percent).await;
    }
}

async fn scaling_up(&self, task: &Task) {
    let running_services = self.node_manager.get_online_all_from_task(&task.get_name()).await;

    if running_services.len() as u64 >= task.get_max_service_count() as u64 {
        return;
    }

    log_info!("[Scaling] Starting new service for {}", task.get_name());

    // NodeManager kümmert sich um alles
    match self.node_manager.start_service(task).await {
        Ok(_) => {
            *self.last_scale_action.write().await = Some(Instant::now());
        }
        Err(e) => {
            log_error!("Scaling start failed: {}", e);
        }
    }
}

async fn scaling_down(
    &self,
    running_services: &[crate::types::ServiceProcessRef],
    task: &Task,
    usage_percent: u64,
) {
    if usage_percent
        < task.get_percent_of_players_to_check_should_auto_stop_the_service() as u64
        && running_services.len() as u64 > task.get_min_service_count()
    {
        // Service mit 0 Spielern finden der lange idle ist
        for service_ref in running_services {
            let s = service_ref.read().await;
            let service = s.get_service();

            if service.get_current_players() == 0 {
                if let Some(idle_since) = service.get_idle_since() {
                    let idle_duration = chrono::Utc::now().naive_utc() - idle_since;
                    if idle_duration.num_seconds() as u64
                        >= task.get_auto_stop_time_by_unused_service_in_seconds()
                    {
                        drop(s); // Lock freigeben vor stop
                        let id = service_ref.get_id().await;

                        match self.node_manager.stop_service(&id, "Auto scaling down").await {
                            Ok(_) => {
                                log_info!("[Scaling] Stopped idle service {}", service.get_name());
                                *self.last_scale_action.write().await = Some(Instant::now());
                                break;
                            }
                            Err(e) => {
                                log_error!("Scaling down failed: {}", e);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn get_db(&self) -> &DatabaseManager {
    self.db.as_ref()
}
} */
