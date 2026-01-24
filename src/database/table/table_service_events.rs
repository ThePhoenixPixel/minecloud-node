use std::sync::Arc;
use serde::{Deserialize, Serialize};

use crate::database::database_manger::*;
use crate::database::db_tools::DbTools;
use crate::error;
use crate::utils::error::CloudError;
use crate::utils::error_kind::CloudErrorKind::*;


const TABLE_SERVICE_EVENTS: &str = "t_service_events";

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TableServiceEvents {
    id: DbInteger,              // service event id
    created_at: DbDateTime,     // format -> YYYY-MM-DD HH:MM:SS
    
    service_uuid: DbString,
    event_type: DbString,
}

impl TableServiceEvents {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn add(&self, db: Arc<dyn DatabaseManager>) -> Result<(), CloudError> {
        db.add_record(TABLE_SERVICE_EVENTS, DbTools::struct_to_db_map(self)?)
            .await
            .map_err(|e| error!(CantCreateDBRecord, e))?;
        Ok(())
    }

    pub fn get_schema() -> Result<Record, CloudError> {
        DbTools::get_schema::<Self>()
    }

    pub async fn check_table(db: &Arc<dyn DatabaseManager>) -> Result<(), CloudError> {
        db.check_table(TABLE_SERVICE_EVENTS, &Self::get_schema()?)
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
    pub fn get_service_uuid(&self) -> DbString {
        self.service_uuid.clone()
    }

    pub fn get_event_type(&self) -> DbString {
        self.event_type.clone()
    }

    // Setter
    pub fn set_service_uuid(&mut self, uuid: DbString) {
        self.service_uuid = uuid;
    }

    pub fn set_event_type(&mut self, event_type: DbString) {
        self.event_type = event_type;
    }

    // Query Methoden
    pub async fn get_by_service_id(
        db: Arc<dyn DatabaseManager>,
        service_id: i64,
    ) -> Result<Vec<Self>, CloudError> {
        let mut filter = Record::new();
        filter.insert("service_id".to_string(), DbValue::Integer(service_id));

        let records = db
            .get_records(TABLE_SERVICE_EVENTS, Some(filter))
            .await
            .map_err(|e| error!(CantDBGetRecords, e))?;

        Self::from_records(records)
    }

    pub async fn get_by_event_type(
        db: Arc<dyn DatabaseManager>,
        event_type: String,
    ) -> Result<Vec<Self>, CloudError> {
        let mut filter = Record::new();
        filter.insert("event_type".to_string(), DbValue::String(event_type));

        let records = db
            .get_records(TABLE_SERVICE_EVENTS, Some(filter))
            .await
            .map_err(|e| error!(CantDBGetRecords, e))?;

        Self::from_records(records)
    }
}