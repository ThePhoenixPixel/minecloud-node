use std::sync::Arc;
use database_manager::{DatabaseManager};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{error, log_info};
use crate::api::internal::PlayerActionRequest;
use crate::utils::error::*;
use crate::types::{Player, PlayerAction, PlayerSession, ServiceRef};
use crate::database::table::{TablePlayerEvents, TablePlayerSessions, TablePlayers};
use crate::manager::service_manager::ServiceManager;
use crate::utils::utils::Utils;

pub struct PlayerManager {
    db_manager: Arc<DatabaseManager>,
    service_manager: Arc<RwLock<ServiceManager>>,
}

impl PlayerManager {
    pub fn new(db_manager: Arc<DatabaseManager>, service_manager: Arc<RwLock<ServiceManager>>) -> PlayerManager {
        PlayerManager {
            db_manager,
            service_manager,
        }
    }

    pub async fn handle_action(&self, req: PlayerActionRequest) -> CloudResult<()> {
        let service_ref = {
            let sm = self.service_manager.read().await;
            sm.get_from_id(&req.get_service_uuid()).await?
        };

        let mut player = self.get_or_create_player(&Player::from(req.get_player_req())).await?;
        let (mut current_players, task) = {
            let s = service_ref.read().await;
            let service = s.get_service();
            (
                service.get_current_players(),
                service.get_task().clone(),
            )
        };

        match req.get_action() {
            PlayerAction::Join => {
                current_players += 1;
                self.on_player_join(&mut player, &service_ref).await?;
                self.add_event(&player, &service_ref, &req.get_action()).await?;
            }
            PlayerAction::Leave => {
                current_players -= 1;
                self.add_event(&player, &service_ref, &req.get_action()).await?;
                self.on_player_leave(&mut player, &service_ref).await?;
            }
        }

        let start_timer = (current_players == 0) || (task.get_percent_of_players_to_check_should_auto_stop_the_service() > (task.get_max_players() * 100) / current_players);

        {
            let mut s = service_ref.write().await;
            s.update_current_player(current_players);

            if start_timer {
                s.start_idle_timer();
            }
        }

        Ok(())
    }

    pub async fn on_player_join(&self, player: &mut Player, service: &ServiceRef) -> CloudResult<()> {
        let id = service.get_id().await;
        if service.is_proxy().await {
            // proxy join handling
            self.create_session(player, &id).await?;
            self.update_last_login(player).await?;

        } else {
            // backend server join handling
            self.update_session(player, &id).await?;

        }

        self.update_last_seen(player).await?;
        Ok(())
    }

    pub async fn on_player_leave(&self, player: &mut Player, service: &ServiceRef) -> CloudResult<()> {
        if service.is_proxy().await {
            // proxy leave handling
            Utils::wait_sec(2).await;
            self.delete_session(player).await?;

        } else {
            // backend server leave handling
            // no special handling required for now
        }

        Ok(())
    }

    async fn register_player(&self, player: &Player) -> CloudResult<Player> {
        let db_player = TablePlayers::new(&player.get_uuid(), &player.get_name())?;
        db_player.create(self.db_manager.as_ref()).await?;
        log_info!(7, "[DB t_players] Register new Player: [{}] [{}]", player.get_name(), player.get_uuid_str());
        self.get_player_by_uuid(&player.get_uuid())
            .await?
            .ok_or_else(|| error!(CantRegisterPlayer))
    }

    async fn get_or_create_player(&self, player: &Player) -> CloudResult<Player> {
        if let Some(mut existing) = self.get_player_by_uuid(&player.get_uuid()).await? {
            self.set_session_for_player(&mut existing).await?;
            return Ok(existing);
        }

        let mut player = self.register_player(player).await?;
        self.set_session_for_player(&mut player).await?;
        Ok(player)
    }

    async fn update_last_seen(&self, player: &Player) -> CloudResult<()> {
        TablePlayers::update_last_seen(self.db_manager.as_ref(), player.get_id()).await?;
        log_info!(8, "[DB t_players] Update 'last_seen' for Player: [{}] [{}]", player.get_name(), player.get_uuid_str());
        Ok(())
    }

    async fn update_last_login(&self, player: &Player) -> CloudResult<()> {
        TablePlayers::update_last_login(self.db_manager.as_ref(), player.get_id()).await?;
        log_info!(8, "[DB t_players] Update 'last_login' for Player: [{}] [{}]", player.get_name(), player.get_uuid_str());
        Ok(())
    }

    async fn get_player_by_uuid(&self, uuid: &Uuid) -> CloudResult<Option<Player>> {
        let player = TablePlayers::find_by_uuid(self.db_manager.as_ref(), uuid)
            .await?
            .map(Player::from);
        Ok(player)
    }

    async fn create_session(&self, player: &mut Player, service_uuid: &Uuid) -> CloudResult<()> {
        let session = TablePlayerSessions::new(player.get_id(), service_uuid);
        session.create(self.db_manager.as_ref()).await?;
        self.set_session_for_player(player).await?;
        log_info!(7, "[DB t_player_sessions] Create Player Session. Player: [{}]", player.get_name());
        Ok(())
    }

    async fn update_session(&self, player: &mut Player, service_uuid: &Uuid) -> CloudResult<()> {
        TablePlayerSessions::update_by_player_id(self.get_db(), player.get_id(), service_uuid).await?;
        self.set_session_for_player(player).await?;
        Ok(())
    }

    async fn delete_session(&self, player: &mut Player) -> CloudResult<()> {
        TablePlayerSessions::delete_by_player_id(self.get_db(), player.get_id()).await?;
        player.clear_session();
        Ok(())
    }

    async fn set_session_for_player(&self, player: &mut Player) -> CloudResult<()> {
        if let Some(s) = TablePlayerSessions::find_by_player_id(self.get_db(), player.get_id()).await? {
            player.set_session(PlayerSession::from(s));
        }
        Ok(())
    }

    pub async fn add_event(&self, player: &Player, service: &ServiceRef, event_type: &PlayerAction) -> CloudResult<()> {
        // Todo: beim zu schnellen join und leave vom proxy in player events session id = null
        let event = TablePlayerEvents::new(player, service, event_type.to_string()).await;
        event.create(self.get_db()).await?;
        Ok(())
    }

    fn get_db(&self) -> &DatabaseManager {
        self.db_manager.as_ref()
    }
}
