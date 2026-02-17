use database_manager::{DatabaseController, Table, TableDerive};
use database_manager::types::{DBDatetime, DBText, DBUInt, DBVarChar, DbResult};

use crate::database::DBTools;
use crate::types::{Player, Service};

#[derive(TableDerive, Debug, Clone, Default)]
#[table_name("t_player_events")]
pub struct TablePlayerEvents {
    #[primary_key]
    #[auto_increment]
    id: DBUInt,                     // session ID
    created_at: DBDatetime,       // format -> YYYY-MM-DD HH:MM:SS

    player_id: DBUInt,
    event_type: DBText,

    #[nullable]
    session_id: Option<DBUInt>,

    service_uuid: DBVarChar,
}

impl TablePlayerEvents {
    pub fn new(player: &Player, service: &Service, event_type: String) -> Self {
        TablePlayerEvents {
            id: Default::default(),
            created_at: DBDatetime::get_now(),
            session_id: player.get_session().map(|s| DBUInt::from(s.get_id())),
            player_id: DBUInt::from(player.get_id()),
            service_uuid: DBTools::uuid_to_varchar(&service.get_id()),
            event_type: DBText::from(event_type),
        }
    }

    pub async fn create<M: DatabaseController>(&self, db: &M) -> DbResult<()> {
        self.insert(db).await?;
        Ok(())
    }
}