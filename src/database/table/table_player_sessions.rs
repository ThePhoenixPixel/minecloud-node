use std::sync::Arc;
use serde::{Deserialize, Serialize};

use crate::database::database_manger::*;
use crate::database::db_tools::DbTools;
use crate::error;
use crate::utils::error::CloudError;
use crate::utils::error_kind::CloudErrorKind::*;


const TABLE_PLAYER_SESSIONS: &str = "t_player_sessions";

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TablePlayerSessions {
    session_id: DbString,    // session UUID
    player_id: i64,
    service_id: i64,
}

impl TablePlayerSessions {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn add(&self, db: Arc<dyn DatabaseManager>) -> Result<(), CloudError> {
        db.add_record(TABLE_PLAYER_SESSIONS, DbTools::struct_to_db_map(self)?)
            .await
            .map_err(|e| error!(CantDBAddRecord, e))?;
        Ok(())
    }

    pub fn get_schema() -> Result<Record, CloudError> {
        DbTools::get_schema::<Self>()
    }

    pub async fn check_table(db: &Arc<dyn DatabaseManager>) -> Result<(), CloudError> {
        db.check_table(TABLE_PLAYER_SESSIONS, &Self::get_schema()?)
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
    pub fn get_session_id(&self) -> &str {
        &self.session_id
    }

    pub fn get_player_id(&self) -> i64 {
        self.player_id
    }

    pub fn get_service_id(&self) -> i64 {
        self.service_id
    }

    // Setter
    pub fn set_session_id(&mut self, session_id: DbString) {
        self.session_id = session_id;
    }

    pub fn set_player_id(&mut self, player_id: i64) {
        self.player_id = player_id;
    }

    pub fn set_service_id(&mut self, service_id: i64) {
        self.service_id = service_id;
    }

    // Query Methoden
    pub async fn get_by_session_id(
        db: Arc<dyn DatabaseManager>,
        session_id: String,
    ) -> Result<Option<Self>, CloudError> {
        let mut filter = Record::new();
        filter.insert("session_id".to_string(), DbValue::String(session_id));

        let records = db
            .get_records(TABLE_PLAYER_SESSIONS, Some(filter))
            .await
            .map_err(|e| error!(CantDBGetRecords, e))?;

        if records.is_empty() {
            return Ok(None);
        }

        Ok(Some(Self::from_record(&records[0])?))
    }

    pub async fn get_by_player_id(
        db: Arc<dyn DatabaseManager>,
        player_id: i64,
    ) -> Result<Vec<Self>, CloudError> {
        let mut filter = Record::new();
        filter.insert("player_id".to_string(), DbValue::Integer(player_id));

        let records = db
            .get_records(TABLE_PLAYER_SESSIONS, Some(filter))
            .await
            .map_err(|e| error!(CantDBGetRecords, e))?;

        Self::from_records(records)
    }
}