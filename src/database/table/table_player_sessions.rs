use database_manager::types::{
    DBDatetime, DBUInt, DBVarChar, DbResult, Filter, QueryFilters, Row, Value,
};
use database_manager::{DatabaseController, Table, TableDerive};
use uuid::Uuid;

use crate::database::DBTools;
use crate::database::table::TableServices;
use crate::types::PlayerSession;

#[derive(TableDerive, Debug, Clone)]
#[table_name("t_player_sessions")]
pub struct TablePlayerSessions {
    #[primary_key]
    #[auto_increment]
    id: DBUInt, // session ID
    created_at: DBDatetime, // format -> YYYY-MM-DD HH:MM:SS

    #[nullable]
    updated_at: Option<DBDatetime>,

    player_id: DBUInt,
    service_uuid: DBVarChar,
}

#[derive(TableDerive, Debug, Clone)]
pub struct TablePlayerSessionsService {
    #[primary_key]
    #[auto_increment]
    id: DBUInt, // session ID
    created_at: DBDatetime, // format -> YYYY-MM-DD HH:MM:SS

    #[nullable]
    updated_at: Option<DBDatetime>,

    player_id: DBUInt,
    service_uuid: DBVarChar,
}

impl TablePlayerSessions {
    pub fn new(player_id: u64, service_uuid: &Uuid) -> Self {
        TablePlayerSessions {
            id: DBUInt::default(),
            created_at: DBDatetime::get_now(),
            updated_at: None,
            player_id: DBUInt::from(player_id),
            service_uuid: DBTools::uuid_to_varchar(service_uuid),
        }
    }

    pub async fn create<M: DatabaseController>(&self, manager: &M) -> DbResult<()> {
        self.insert(manager).await?;
        Ok(())
    }

    pub async fn update_by_player_id<M: DatabaseController>(
        manager: &M,
        id: u64,
        service_uuid: &Uuid,
    ) -> DbResult<()> {
        manager
            .update(
                Self::table_name(),
                &QueryFilters::new().add(Filter::eq("player_id", Value::from(id))),
                &Row::from([
                    (
                        "updated_at".to_string(),
                        Value::DateTime(DBDatetime::get_now()),
                    ),
                    (
                        "service_uuid".to_string(),
                        DBTools::uuid_to_value(service_uuid),
                    ),
                ]),
            )
            .await?;
        Ok(())
    }

    pub async fn delete_by_player_id<M: DatabaseController>(manager: &M, id: u64) -> DbResult<()> {
        manager
            .delete(
                Self::table_name(),
                &QueryFilters::new().add(Filter::eq("player_id", Value::from(id))),
            )
            .await?;
        Ok(())
    }

    pub async fn find_by_player_id<M: DatabaseController>(
        db: &M,
        id: u64,
    ) -> DbResult<Option<TablePlayerSessions>> {
        let mut filter = QueryFilters::new();
        filter.add_filter(Filter::eq("player_id", Value::from(id)));

        let row = db
            .query_one(TablePlayerSessions::table_name(), &filter)
            .await?;

        if let Some(row) = row {
            Ok(Some(Self::from_row(&row)?))
        } else {
            Ok(None)
        }
    }

    pub async fn count_players_from_task<M: DatabaseController>(
        db: &M,
        task_name: &String,
    ) -> DbResult<u64> {
        let table_name = format!("{} ps", Self::table_name());
        let z1 = format!("{}.service_uuid", TablePlayerSessions::table_name());
        let z2 = format!("{}.uuid", TableServices::table_name());
        let f = QueryFilters::new().add(Filter::eq(
            format!("{}.task", TableServices::table_name()),
            Value::from(task_name.to_string()),
        ));
        let rows = db
            .query_with_join(&table_name, vec![(TableServices::table_name(), z1, z2)], &f)
            .await?;

        Ok(rows.len() as u64)
    }

    pub fn get_id(&self) -> u64 {
        self.id.0
    }
}

impl From<TablePlayerSessions> for PlayerSession {
    fn from(value: TablePlayerSessions) -> Self {
        PlayerSession::new(
            value.get_id(),
            Uuid::parse_str(value.service_uuid.value().as_ref()).unwrap(),
        )
    }
}
