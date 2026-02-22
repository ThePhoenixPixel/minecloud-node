use database_manager::types::*;
use database_manager::{DatabaseController, Table, TableDerive};
use uuid::Uuid;

use crate::database::DBTools;
use crate::types::Player;
use crate::utils::error::*;
use crate::utils::utils::Utils;

pub const UUID_LENGTH: usize = 36;

#[derive(TableDerive, Debug, Clone)]
#[table_name("t_players")]
pub struct TablePlayers {
    #[primary_key]
    #[auto_increment]
    id: DBUInt, // player ID

    created_at: DBDatetime, // format -> YYYY-MM-DD HH:MM:SS

    uuid: DBVarChar,
    name: DBText,

    #[nullable]
    last_seen: Option<DBDatetime>, // format -> YYYY-MM-DD HH:MM:SS

    #[nullable]
    last_login: Option<DBDatetime>, // format -> YYYY-MM-DD HH:MM:SS
}

impl TablePlayers {
    pub fn new(uuid: &Uuid, name: &str) -> CloudResult<Self> {
        Ok(Self {
            id: DBUInt::default(),
            created_at: DBDatetime::get_now(),
            uuid: DBTools::uuid_to_varchar(uuid),
            name: DBText::from(name),
            last_seen: None,
            last_login: None,
        })
    }

    pub async fn update_last_login<M: DatabaseController>(manager: &M, id: u64) -> DbResult<usize> {
        manager
            .update(
                Self::table_name(),
                &QueryFilters::new().add(Filter::eq("id", Value::from(id))),
                &Row::from([(
                    "last_login".into(),
                    Value::DateTime(Utils::get_datetime_now().into()),
                )]),
            )
            .await
    }

    pub async fn update_last_seen<M: DatabaseController>(manager: &M, id: u64) -> DbResult<usize> {
        manager
            .update(
                Self::table_name(),
                &QueryFilters::new().add(Filter::eq("id", Value::UInt(id.into()))),
                &Row::from([(
                    "last_seen".into(),
                    Value::DateTime(Utils::get_datetime_now().into()),
                )]),
            )
            .await
    }

    pub async fn find_by_uuid<M: DatabaseController>(
        manager: &M,
        uuid: &Uuid,
    ) -> DbResult<Option<Self>> {
        let row = manager
            .query_one(
                Self::table_name(),
                &QueryFilters::new().add(Filter::eq("uuid", DBTools::uuid_to_value(uuid))),
            )
            .await?;

        if let Some(row) = row {
            Ok(Some(Self::from_row(&row)?))
        } else {
            Ok(None)
        }
    }

    pub async fn create<M: DatabaseController>(&self, manager: &M) -> DbResult<()> {
        self.insert(manager).await?;
        Ok(())
    }

    // Getter
    pub fn get_id(&self) -> DBUInt {
        self.id.clone()
    }
    pub fn get_uuid(&self) -> DBVarChar {
        self.uuid.clone()
    }

    pub fn get_name(&self) -> DBText {
        self.name.clone()
    }

    // Setter
    pub fn set_uuid(&mut self, uuid: &DBVarChar) {
        self.uuid = uuid.clone();
    }

    pub fn set_name(&mut self, name: &DBText) {
        self.name = name.clone();
    }

    pub fn set_last_seen(&mut self, last_seen: &DBDatetime) {
        self.last_seen = Some(last_seen.clone());
    }

    pub fn set_last_login(&mut self, last_login: &DBDatetime) {
        self.last_login = Some(last_login.clone());
    }
}

impl From<TablePlayers> for Player {
    fn from(table: TablePlayers) -> Player {
        Player::new(
            table.id.0,
            table.name.0,
            Uuid::parse_str(table.uuid.value.as_ref()).unwrap(),
            None,
        )
    }
}
