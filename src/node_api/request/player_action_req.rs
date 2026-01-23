use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::cloud::Cloud;
use crate::error;
use crate::utils::error::CloudError;
use crate::utils::error_kind::CloudErrorKind::*;
use crate::utils::player::Player;
use crate::utils::player_action::PlayerAction;

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerActionRequest {
    action: PlayerAction,
    service_id: Uuid,
    player: Player,
}

impl PlayerActionRequest {
    pub async fn execute(&self, cloud: Arc<RwLock<Cloud>>) -> Result<(), CloudError> {
        let mut service = {
            let cloud = cloud.read().await;
            match cloud.get_local().get_from_id(&self.service_id) {
                Some(service) => service,
                None => return Err(error!(CantFindServiceFromUUID)),
            }
        };

        match self.action {
            PlayerAction::Join => self.join(cloud).await,
            PlayerAction::Leave => self.leave(cloud).await,
        }
        Ok(())
    }

    async fn join(&self, cloud: Arc<RwLock<Cloud>>) {}

    async fn leave(&self, cloud: Arc<RwLock<Cloud>>) {}
}
