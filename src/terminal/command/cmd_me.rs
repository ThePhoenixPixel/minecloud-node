use crate::api::internal::{OutgoingMessage, OutgoingMessageType, PlayerActionMessage};
use crate::cloud::Cloud;
use crate::log_info;
use crate::types::PlayerAction;
use crate::utils::error::CloudResult;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct CmdMe;

impl CmdMe {
    pub(crate) async fn execute(cloud: Arc<RwLock<Cloud>>, args: Vec<&str>) -> CloudResult<()> {

        let player_name = match args.get(1) {
            Some(arg) => *arg,
            None => {
                log_info!("kein player name?");
                return Ok(());
            }
        };

        let server_name = match args.get(2) {
            Some(arg) => *arg,
            None => {
                log_info!("Kein server name??");
                return Ok(());
            }
        };

        let player = {
            match cloud.read().await.get_player_manager().find_player_by_name(player_name).await? {
                Some(p) => p,
                None => {
                    log_info!("Player nicht gefunden!");
                    return Ok(());
                }
            }
        };
        let service = {
            match cloud.read().await.get_node_manager().get_service_manager().read().await.filter_services(|s| s.get_name() == server_name).await.get(0).cloned() {
                Some(s) => s,
                None => {
                    log_info!("Kein service gefunden!");
                    return Ok(());
                }
            }
        };

        let player_act_msg = PlayerActionMessage::new(PlayerAction::SwitchServer, service.get_id().await, service.get_name().await, player.get_uuid(), player.get_name().into());
        let msg = OutgoingMessage::ok(None, OutgoingMessageType::ConnectPlayerToServer, serde_json::to_value(&player_act_msg).ok().unwrap());

        let proxys = {
            cloud.read().await.get_node_manager().get_service_manager().read().await.filter_services(|s| s.is_proxy() && s.is_running()).await
        };

        for proxy in proxys {
            let mut pr = proxy.write().await;
            if pr.send(&msg).await {
                log_info!("Send actiont o proxy: {}", pr.get_name())
            } else  {
                log_info!("Cant Send actiont o proxy: {}", pr.get_name())
            }
        }

        Ok(())
    }

    fn tab_complete(_args: Vec<&str>) -> Vec<String> {
        todo!()
    }
}

fn print_info(process: &sysinfo::Process) {
    log_info!("------------>Cloud Info<------------");
    log_info!("Cpu: {:.2}%", process.cpu_usage());
    log_info!("Ram {} Bytes", process.memory());
    log_info!("------------------------------------");
}
