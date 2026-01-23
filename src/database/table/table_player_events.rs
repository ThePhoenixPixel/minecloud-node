use std::sync::Arc;
use serde::{Deserialize, Serialize};

use crate::database::database_manger::*;
use crate::database::db_tools::DbTools;
use crate::error;
use crate::utils::error::CloudError;
use crate::utils::error_kind::CloudErrorKind::*;


const TABLE_PLAYER_EVENTS: &str = "t_player_events";

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TablePlayerEvents {
    player_id: i64,
    service_id: i64,
    session_id: DbString,
    event_type: DbString,
    created_at: DbString,
}

impl TablePlayerEvents {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn add(&self, db: Arc<dyn DatabaseManager>) -> Result<(), CloudError> {
        db.add_record(TABLE_PLAYER_EVENTS, DbTools::struct_to_db_map(self)?)
            .await
            .map_err(|e| error!(CantDBAddRecord, e))?;
        Ok(())
    }

    pub fn get_schema() -> Result<Record, CloudError> {
        DbTools::get_schema::<Self>()
    }

    pub async fn check_table(db: &Arc<dyn DatabaseManager>) -> Result<(), CloudError> {
        db.check_table(TABLE_PLAYER_EVENTS, &Self::get_schema()?)
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
    pub fn get_player_id(&self) -> i64 {
        self.player_id
    }

    pub fn get_service_id(&self) -> i64 {
        self.service_id
    }

    pub fn get_session_id(&self) -> &str {
        &self.session_id
    }

    pub fn get_event_type(&self) -> &str {
        &self.event_type
    }

    pub fn get_created_at(&self) -> &str {
        &self.created_at
    }

    // Setter
    pub fn set_player_id(&mut self, player_id: i64) {
        self.player_id = player_id;
    }

    pub fn set_service_id(&mut self, service_id: i64) {
        self.service_id = service_id;
    }

    pub fn set_session_id(&mut self, session_id: DbString) {
        self.session_id = session_id;
    }

    pub fn set_event_type(&mut self, event_type: DbString) {
        self.event_type = event_type;
    }

    pub fn set_created_at(&mut self, created_at: DbString) {
        self.created_at = created_at;
    }

    // Query Methoden
    pub async fn get_by_player_id(
        db: Arc<dyn DatabaseManager>,
        player_id: i64,
    ) -> Result<Vec<Self>, CloudError> {
        let mut filter = Record::new();
        filter.insert("player_id".to_string(), DbValue::Integer(player_id));

        let records = db
            .get_records(TABLE_PLAYER_EVENTS, Some(filter))
            .await
            .map_err(|e| error!(CantDBGetRecords, e))?;

        Self::from_records(records)
    }

    pub async fn get_by_session_id(
        db: Arc<dyn DatabaseManager>,
        session_id: String,
    ) -> Result<Vec<Self>, CloudError> {
        let mut filter = Record::new();
        filter.insert("session_id".to_string(), DbValue::String(session_id));

        let records = db
            .get_records(TABLE_PLAYER_EVENTS, Some(filter))
            .await
            .map_err(|e| error!(CantDBGetRecords, e))?;

        Self::from_records(records)
    }
}