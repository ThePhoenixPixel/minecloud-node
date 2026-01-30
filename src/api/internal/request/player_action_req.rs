use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::cloud::Cloud;
use crate::types::service::Service;
use crate::database::manager::DatabaseManager;
use crate::error;
use crate::utils::error::CloudError;
use crate::utils::error_kind::CloudErrorKind::*;
use crate::utils::player::Player;
use crate::utils::player_action::PlayerAction;
use crate::utils::utils::Utils;

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerActionRequest {
    action: PlayerAction,
    service_uuid: Uuid,
    player: Player,
}

impl PlayerActionRequest {
    pub async fn execute(&self, cloud: Arc<RwLock<Cloud>>) -> Result<(), CloudError> {
        let (service, db) = {
            let cloud = cloud.read().await;
            let service = match cloud.get_local().get_from_id(&self.service_uuid) {
                Some(service) => service,
                None => return Err(error!(CantFindServiceFromUUID)),
            };
            let db = cloud.get_database_manager();
            (service, db)
        };

        match self.action {
            PlayerAction::Join => self.join(&service, &db).await?,
            PlayerAction::Leave => self.leave(&service, &db).await?,
        }
        Ok(())
    }

    async fn join(&self, service: &Service, db: &DatabaseManager) -> Result<(), CloudError> {
        if service.is_proxy() {
            self.player.add_to_db(&db).await?;
            self.player.add_session(&db, service).await?;
        } else {
            self.player.update_session(db, service).await?;
        }
        Utils::wait_nano(500).await;
        self.player.add_event(&db, &self.action, &service).await?;
        Ok(())
    }

    async fn leave(&self, service: &Service, db: &DatabaseManager) -> Result<(), CloudError> {
        self.player.add_event(&db, &self.action, &service).await?;
        if service.is_proxy() {
            Utils::wait_nano(500).await;
            self.player.delete_session(&db).await?
        }
        Ok(())
    }

}
