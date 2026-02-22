use crate::utils::error::CloudResult;
use database_manager::{DatabaseController, Table};

pub use table_player_events::*;
pub use table_player_sessions::*;
pub use table_players::*;
pub use table_services::TableServices;

mod table_player_events;
mod table_player_sessions;
mod table_players;
//mod table_service_events;
mod table_services;

pub struct Tables;
impl Tables {
    pub async fn check_tables<M: DatabaseController>(manager: &M) -> CloudResult<()> {
        TablePlayers::sync(manager).await?;
        TablePlayerSessions::sync(manager).await?;
        TablePlayerEvents::sync(manager).await?;
        TableServices::sync(manager).await?;

        Ok(())
    }
}
