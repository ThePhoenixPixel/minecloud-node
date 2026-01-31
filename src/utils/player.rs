use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::service::Service;
use crate::database::manager::DatabaseManager;
use crate::database::table::table_player_sessions::TablePlayerSessions;
use crate::database::table::table_players::TablePlayers;
use crate::{log_info, log_warning};
use crate::database::table::table_player_events::TablePlayerEvents;
use crate::utils::error::cloud_error::CloudError;
use crate::utils::player_action::PlayerAction;
use crate::utils::utils::Utils;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct Player {
    uuid: Uuid,
    name: String,
}

impl Player {
    pub fn get_uuid(&self) -> Uuid {
        self.uuid
    }
    
    pub fn get_uuid_str(&self) -> String {
        self.uuid.to_string()
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    pub async fn add_to_db(&self, db: &DatabaseManager) -> Result<(), CloudError> {
        let mut player = TablePlayers::new();
        player.set_name(&self.name);
        player.set_uuid(&self.get_uuid_str());
        player.add(db).await?;
        log_info!(7, "Create Player to DB [{}] [{}]", self.get_name(), self.get_uuid_str());
        Ok(())
    }

    pub async fn add_session(&self, db: &DatabaseManager, _service: &Service) -> Result<(), CloudError> {
        let mut session = TablePlayerSessions::new();
        session.set_player_uuid(self.get_uuid_str());
        session.add(&db).await?;
        self.update_last_login(&db).await?;
        log_info!(7, "Create Player Session. Player: [{}] Session-ID: [{}]", self.get_name(), session.get_id());
        Ok(())
    }

    pub async fn add_event(&self, db: &DatabaseManager, event_type: &PlayerAction, service: &Service) -> Result<(), CloudError> {
        let mut event = TablePlayerEvents::new();
        event.set_session_id(match TablePlayerSessions::get_by_player_uuid(&db, self.get_uuid_str()).await? {
            Some(s) => s.get_id(),
            None => {
                log_warning!(5, "Cant find Session ID for Player: [{}] [{}]", self.get_name(), self.get_uuid_str());
                0
            },
        });
        
        event.set_event_type(event_type.to_string());
        event.set_player_uuid(self.get_uuid_str());
        event.set_service_uuid(service.get_id().to_string());
        event.add(&db).await?;
        
        self.update_last_seen(&db).await?;
        Ok(())
    }

    pub async fn update_last_seen(&self, db: &DatabaseManager) -> Result<(), CloudError> {
        let mut player = match TablePlayers::get_by_uuid(&db, self.get_uuid_str()).await? {
            Some(player) => player,
            None => {
                log_warning!(5, "[Upd t_players 'last_seen'] Cant find Player in Database [{}] [{}]", self.get_name(), self.get_uuid_str());
                return Ok(());
            },
        };

        player.set_last_seen(&Utils::get_datetime_now());
        player.update(&db).await?;
        log_info!(8, "[Upd t_players 'last_seen'] updated for Player: [{}] [{}]", self.get_name(), self.get_uuid_str());
        Ok(())
    }

    pub async fn update_last_login(&self, db: &DatabaseManager) -> Result<(), CloudError> {
        let mut player = match TablePlayers::get_by_uuid(&db, self.get_uuid_str()).await? {
            Some(player) => player,
            None => {
                log_warning!(6, "[Upd t_players 'last_login'] Cant find Player in Database [{}] [{}]", self.get_name(), self.get_uuid_str());
                return Ok(());
            },
        };

        player.set_last_login(&Utils::get_datetime_now());
        player.update(&db).await?;
        log_info!(7, "[Upd t_players 'last_login'] updated for Player: [{}] [{}]", self.get_name(), self.get_uuid_str());
        Ok(())
    }

    pub async fn update_session(&self, db: &DatabaseManager, service: &Service) -> Result<(), CloudError> {
        match TablePlayerSessions::get_by_player_uuid(&db, self.get_uuid_str()).await? {
            Some(mut session) => {
                // find a Session -> update
                session.set_service_uuid(service.get_id().to_string());
                session.update(&db, session.get_id()).await?;
            }
            None => {
                // cant find Session -> Create
                self.add_session(&db, &service).await?;
            }
        }
        Ok(())
    }

    pub async fn delete_session(&self, db: &DatabaseManager) -> Result<(), CloudError> {
        match TablePlayerSessions::get_by_player_uuid(&db, self.get_uuid_str()).await? {
            Some(session) => {
                // find a Session -> delete
                session.delete_from_player_uuid(&db).await?;
                log_info!(7, "Session Deleted for Player: [{}]", self.name)
            }
            None => log_warning!(6, "Cant Find Session for Player [{}] to delete", self.name),
        }
        Ok(())
    }

}
