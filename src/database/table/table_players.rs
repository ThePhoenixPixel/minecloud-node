use std::sync::Arc;
use serde::{Deserialize, Serialize};

use crate::database::database_manger::*;
use crate::database::db_tools::DbTools;
use crate::error;
use crate::utils::error::CloudError;
use crate::utils::error_kind::CloudErrorKind::*;

const TABLE_PLAYERS: &str = "t_players";

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TablePlayers {
    uuid: DbString,
    name: DbString,
    last_seen: DbString,
    last_login: DbString,
}

impl TablePlayers {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn add(&self, db: Arc<dyn DatabaseManager>) -> Result<(), CloudError> {
        db.add_record(TABLE_PLAYERS, DbTools::struct_to_db_map(self)?)
            .await
            .map_err(|e| error!(CantDBAddRecord, e))?;
        Ok(())
    }

    pub fn get_schema() -> Result<Record, CloudError> {
        DbTools::get_schema::<Self>()
    }

    pub async fn check_table(db: &Arc<dyn DatabaseManager>) -> Result<(), CloudError> {
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
    pub fn set_uuid(&mut self, uuid: DbString) {
        self.uuid = uuid;
    }

    pub fn set_name(&mut self, name: DbString) {
        self.name = name;
    }

    pub fn set_last_seen(&mut self, last_seen: DbString) {
        self.last_seen = last_seen;
    }

    pub fn set_last_login(&mut self, last_login: DbString) {
        self.last_login = last_login;
    }

    // Query Methoden
    pub async fn get_by_uuid(
        db: Arc<dyn DatabaseManager>,
        uuid: String,
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
        db: Arc<dyn DatabaseManager>,
        name: String,
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