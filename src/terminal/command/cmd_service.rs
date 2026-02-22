use std::io::{Error, ErrorKind};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cloud::Cloud;
use crate::log_info;
use crate::terminal::command_manager::CommandManager;
use crate::types::Service;

pub struct CmdService;

impl CommandManager for CmdService {
    async fn execute(cloud: Arc<RwLock<Cloud>>, args: Vec<&str>) -> Result<(), Error> {
        let arg1 = match args.get(1) {
            Some(arg1) => *arg1,
            None => {
                return Err(Error::new(
                    ErrorKind::Other,
                    "bitte gebe ein argument an -> list /  an".to_string(),
                ));
            }
        };

        match arg1 {
            "list" => list(cloud.clone(), args).await,
            "reload" => reload(cloud.clone()).await,
            _ => Err(Error::new(
                ErrorKind::Other,
                "bitte gebe ein gültiges argument an -> list /  an".to_string(),
            )),
        }
    }
    fn tab_complete(_args: Vec<&str>) -> Vec<String> {
        todo!()
    }
}

async fn reload(_cloud: Arc<RwLock<Cloud>>) -> Result<(), Error> {
    /*
    let services = cloud.lock().await.get_all().get_all().await;

    for service in services {
        service.connect_to_network().await?;
    }
    log_info!("Service neu reloaded");

     */

    //Ok(())
    todo!();
}

async fn list(cloud: Arc<RwLock<Cloud>>, args: Vec<&str>) -> Result<(), Error> {
    let service_manager = {
        let cloud_guard = cloud.read().await;
        cloud_guard.get_node_manager()
    };
    todo!()
    /*
    let services = service_manager
        .read()
        .await
        .get_all()
        .iter()
        .map(async |s| s.read().await.get_service().clone())
        .collect();
    let arg2 = match args.get(2) {
        Some(arg2) => *arg2,
        None => return list_all(&services),
    };

    match arg2 {
        "--online" => list_online(&services),
        "--on" => list_online(&services),
        _ => Err(Error::new(
            ErrorKind::Other,
            "bitte gebe einen gültigen para an -> --online, --on".to_string(),
        )),
    }*/
}

fn list_online(services: &Vec<Service>) -> Result<(), Error> {
    log_info!("Dies sind alle Online Services:");
    log_info!("Name | Server Address | Plugin Listener");
    for service in services {
        if !service.is_start() {
            continue;
        }

        log_info!(
            "{} | {} | {}",
            service.get_name(),
            service.get_server_listener().to_string(),
            service.get_plugin_listener().to_string()
        );
    }
    Ok(())
}

fn list_all(services: &Vec<Service>) -> Result<(), Error> {
    log_info!("Name | Server Address | Plugin Listener | Online");
    for service in services {
        log_info!(
            "{} | {} | {} | {} ",
            service.get_name(),
            service.get_server_listener().to_string(),
            service.get_plugin_listener().to_string(),
            service.is_start()
        );
    }
    Ok(())
}
