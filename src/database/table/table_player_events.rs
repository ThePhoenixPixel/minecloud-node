use serde::{Deserialize, Serialize};

use crate::database::manager::*;
use crate::database::db_tools::DbTools;
use crate::database::db_types::*;
use crate::error;
use crate::utils::error::cloud_error::CloudError;
use crate::utils::error::error_kind::CloudErrorKind::*;
use crate::utils::utils::Utils;

const TABLE_PLAYER_EVENTS: &str = "t_player_events";

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TablePlayerEvents {
    id: DbInteger,                // player event ID
    created_at: DbDateTime,       // format -> YYYY-MM-DD HH:MM:SS

    session_id: DbInteger,
    player_uuid: DbString,
    service_uuid: DbString,
    event_type: DbString,
}

impl TablePlayerEvents {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn add(&mut self, db: &DatabaseManager) -> Result<(), CloudError> {
        self.created_at = Utils::get_datetime_now();
        db.add_record(TABLE_PLAYER_EVENTS, DbTools::struct_to_db_map(self)?)
            .await
            .map_err(|e| error!(CantCreateDBRecord, e))?;
        Ok(())
    }

    pub fn get_schema() -> Result<Record, CloudError> {
        DbTools::get_schema::<Self>()
    }

    pub async fn check_table(db: &DatabaseManager) -> Result<(), CloudError> {
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
    pub fn get_player_uuid(&self) -> DbString {
        self.player_uuid.clone()
    }

    pub fn get_service_uuid(&self) -> DbString {
        self.service_uuid.clone()
    }

    pub fn get_session_id(&self) -> DbInteger {
        self.session_id
    }

    pub fn get_event_type(&self) -> DbString {
        self.event_type.clone()
    }

    // Setter
    pub fn set_player_uuid(&mut self, uuid: DbString) {
        self.player_uuid = uuid;
    }

    pub fn set_service_uuid(&mut self, uuid: DbString) {
        self.service_uuid = uuid;
    }

    pub fn set_session_id(&mut self, id: DbInteger) {
        self.session_id = id;
    }

    pub fn set_event_type(&mut self, event_type: DbString) {
        self.event_type = event_type;
    }

    pub async fn get_by_player_id(
        db: &DatabaseManager,
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
        db: &DatabaseManager,
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