use database_manager::types::{DBDatetime, DBText, DBUInt, DBVarChar, DbResult};
use database_manager::{DatabaseController, Table, TableDerive};
use uuid::Uuid;
use crate::database::DBTools;
use crate::types::{Player, ServiceProcessRef};

#[derive(TableDerive, Debug, Clone)]
#[table_name("t_player_events")]
pub struct TablePlayerEvents {
    #[primary_key]
    #[auto_increment]
    id: DBUInt, // session ID
    created_at: DBDatetime, // format -> YYYY-MM-DD HH:MM:SS

    player_id: DBUInt,
    event_type: DBText,

    #[nullable]
    session_id: Option<DBUInt>,

    service_uuid: DBVarChar,
}

impl TablePlayerEvents {
    pub async fn new(player: &Player, service: &ServiceProcessRef, event_type: String) -> Self {
        Self::new_with_session(player, service, event_type, None).await
    }

    pub async fn new_with_session(
        player: &Player,
        service: &ServiceProcessRef,
        event_type: String,
        session_id_override: Option<u64>,
    ) -> Self {
        let session_id = session_id_override
            .map(|id| DBUInt::from(id))
            .or_else(|| player.get_session().clone().map(|s| DBUInt::from(s.get_id())));

        TablePlayerEvents {
            id: Default::default(),
            created_at: DBDatetime::get_now(),
            player_id: DBUInt::from(player.get_id()),
            service_uuid: DBTools::uuid_to_varchar(&service.get_id().await),
            event_type: DBText::from(event_type),
            session_id,
        }
    }

    pub async fn create<M: DatabaseController>(&self, db: &M) -> DbResult<()> {
        self.insert(db).await?;
        Ok(())
    }
}
