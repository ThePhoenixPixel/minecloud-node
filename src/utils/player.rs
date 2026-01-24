use crate::utils::logger::Logger;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::service::Service;
use crate::database::database_manger::DatabaseManager;
use crate::database::table::table_player_sessions::TablePlayerSessions;
use crate::database::table::table_players::TablePlayers;
use crate::{log_info, log_warning};
use crate::database::table::table_player_events::TablePlayerEvents;
use crate::utils::error::CloudError;
use crate::utils::player_action::PlayerAction;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct Player {
    uuid: Uuid,
    name: String,
}

impl Player {
    pub fn get_uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    pub async fn add_to_db(&self, db: &Arc<dyn DatabaseManager>) -> Result<(), CloudError> {
        let mut player = TablePlayers::new();
        player.set_name(&self.name);
        player.set_uuid(&self.uuid.to_string());
        player.add(db).await
    }

    pub async fn add_session(&self, db: &Arc<dyn DatabaseManager>, service: &Service) -> Result<(), CloudError> {
        let mut session = TablePlayerSessions::new();
        session.set_player_uuid(self.uuid.to_string());
        session.add(&db).await?;
        Ok(())
    }

    pub async fn add_event(&self, db: &Arc<dyn DatabaseManager>, event_type: &PlayerAction, service: &Service) -> Result<(), CloudError> {
        let mut event = TablePlayerEvents::new();
        event.set_session_id(match TablePlayerSessions::get_by_player_uuid(&db, self.uuid.to_string()).await? {
            Some(s) => s.get_id(),
            None => 0,
        });
        
        event.set_event_type(event_type.to_string());
        event.set_player_uuid(self.uuid.to_string());
        event.set_service_uuid(service.get_id().to_string());
        event.add(&db).await?;
        Ok(())
    }

    pub async fn update_session(&self, db: &Arc<dyn DatabaseManager>, service: &Service) -> Result<(), CloudError> {
        match TablePlayerSessions::get_by_player_uuid(&db, self.uuid.to_string()).await? {
            Some(mut session) => {
                // find a Session -> update
                session.set_service_uuid(service.get_id().to_string());
                session.update(&db, session.get_id()).await?;
            }
            None => {
                // cant find Session -> Create
                let mut session = TablePlayerSessions::new();
                session.set_service_uuid(service.get_id().to_string());
                session.set_player_uuid(self.uuid.to_string());
                session.add(&db).await?;
            }
        }
        Ok(())
    }
    pub async fn delete_session(&self, db: &Arc<dyn DatabaseManager>) -> Result<(), CloudError> {
        match TablePlayerSessions::get_by_player_uuid(&db, self.uuid.to_string()).await? {
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
