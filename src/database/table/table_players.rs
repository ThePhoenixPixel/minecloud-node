use serde::{Deserialize, Serialize};

use crate::database::manager::*;
use crate::database::db_tools::DbTools;
use crate::database::db_types::*;
use crate::error;
use crate::utils::error::cloud_error::CloudError;
use crate::utils::error::error_kind::CloudErrorKind::*;
use crate::utils::utils::Utils;

const TABLE_PLAYERS: &str = "t_players";

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TablePlayers {
    id: DbInteger,              // player ID
    created_at: DbDateTime,     // format -> YYYY-MM-DD HH:MM:SS

    uuid: DbString,
    name: DbString,
    last_seen: DbDateTime,      // format -> YYYY-MM-DD HH:MM:SS
    last_login: DbDateTime,     // format -> YYYY-MM-DD HH:MM:SS
}

impl TablePlayers {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn add(&mut self, db: &DatabaseManager) -> Result<(), CloudError> {
        if TablePlayers::get_by_uuid(&db, self.uuid.clone()).await?.is_none() {
            self.created_at = Utils::get_datetime_now();
            self.id = db.add_record(TABLE_PLAYERS, DbTools::struct_to_db_map(self)?)
                .await
                .map_err(|e| error!(CantCreateDBRecord, e))?;
        }
        Ok(())
    }

    pub async fn delete(&self, db: &DatabaseManager, id: DbInteger) -> Result<(), CloudError> {
        db.delete_record(TABLE_PLAYERS, id).await.map_err(|e| error!(CantDeleteDBRecord, e))
    }

    pub async fn update(&self, db: &DatabaseManager) -> Result<(), CloudError> {
        db.update_record(TABLE_PLAYERS, self.id, DbTools::struct_to_db_map(&self)?)
            .await
            .map_err(|e| error!(CantUpdateDBRecord, e))
    }

    pub fn get_schema() -> Result<Record, CloudError> {
        DbTools::get_schema::<Self>()
    }

    pub async fn check_table(db: &DatabaseManager) -> Result<(), CloudError> {
        db.check_table(TABLE_PLAYERS, &Self::get_schema()?)
            .await
            .map_err(|e| error!(CantCreateTable, e))?;
        Ok(())
    }

    pub fn from_record(record: &Record) -> Result<Self, CloudError> {
        let json = DbTools::record_to_json(record);
        serde_json::from_value(json)
            .map_err(|e| error!(DeserializationError, e))
    }

    pub fn from_records(records: Vec<Record>) -> Result<Vec<Self>, CloudError> {
        records.iter().map(Self::from_record).collect()
    }

    // Getter
    pub fn get_uuid(&self) -> &str {
        &self.uuid
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_last_seen(&self) -> &str {
        &self.last_seen
    }

    pub fn get_last_login(&self) -> &str {
        &self.last_login
    }

    // Setter
    pub fn set_uuid(&mut self, uuid: &DbString) {
        self.uuid = uuid.clone();
    }

    pub fn set_name(&mut self, name: &DbString) {
        self.name = name.clone();
    }

    pub fn set_last_seen(&mut self, last_seen: &DbString) {
        self.last_seen = last_seen.clone();
    }

    pub fn set_last_login(&mut self, last_login: &DbString) {
        self.last_login = last_login.clone();
    }

    // Query Methoden
    pub async fn get_by_uuid(
        db: &DatabaseManager,
        uuid: DbString,
    ) -> Result<Option<Self>, CloudError> {
        let mut filter = Record::new();
        filter.insert("uuid".to_string(), DbValue::String(uuid));

        let records = db
            .get_records(TABLE_PLAYERS, Some(filter))
            .await
            .map_err(|e| error!(CantDBGetRecords, e))?;

        if records.is_empty() {
            return Ok(None);
        }

        Ok(Some(Self::from_record(&records[0])?))
    }

    pub async fn get_by_name(
        db: &DatabaseManager,
        name: DbString,
    ) -> Result<Option<Self>, CloudError> {
        let mut filter = Record::new();
        filter.insert("name".to_string(), DbValue::String(name));

        let records = db
            .get_records(TABLE_PLAYERS, Some(filter))
            .await
            .map_err(|e| error!(CantDBGetRecords, e))?;

        if records.is_empty() {
            return Ok(None);
        }

        Ok(Some(Self::from_record(&records[0])?))
    }
}