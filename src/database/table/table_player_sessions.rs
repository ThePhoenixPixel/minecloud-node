use serde::{Deserialize, Serialize};

use crate::database::db_tools::DbTools;
use crate::database::db_types::*;
use crate::database::manager::DatabaseManager;
use crate::error;
use crate::utils::error::cloud_error::CloudError;
use crate::utils::error::error_kind::CloudErrorKind::*;
use crate::utils::utils::Utils;

const TABLE_PLAYER_SESSIONS: &str = "t_player_sessions";

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TablePlayerSessions {
    id: DbInteger,              // session ID
    created_at: DbDateTime,     // format -> YYYY-MM-DD HH:MM:SS
    updated_at: DbDateTime,

    player_uuid: DbString,
    service_uuid: DbString,
}

impl TablePlayerSessions {
    pub fn new() -> Self {
        Self::default()
    }


    pub async fn delete_from_player_uuid(&self, db: &DatabaseManager) -> Result<(), CloudError> {
        match TablePlayerSessions::get_by_player_uuid(&db, self.player_uuid.to_string()).await? {
            Some(session) => {
                session.delete(&db, session.id).await
            }
            None => {
                Ok(())
            }
        }
    }

    pub async fn delete(&self, db: &DatabaseManager, id: DbInteger) -> Result<(), CloudError> {
        db.delete_record(TABLE_PLAYER_SESSIONS, id).await.map_err(|e| error!(CantDeleteDBRecord, e))
    }

    pub async fn update(&self, db: &DatabaseManager, id: DbInteger) -> Result<(), CloudError> {
        db.update_record(TABLE_PLAYER_SESSIONS, id, DbTools::struct_to_db_map(&self)?)
            .await
            .map_err(|e| error!(CantUpdateDBRecord, e))
    }

    pub async fn add(&mut self, db: &DatabaseManager) -> Result<(), CloudError> {
        let time = Utils::get_datetime_now();
        self.updated_at = time.clone();
        self.created_at = time;
        self.id = db.add_record(TABLE_PLAYER_SESSIONS, DbTools::struct_to_db_map(self)?)
            .await
            .map_err(|e| error!(CantCreateDBRecord, e))? as DbInteger;
        Ok(())
    }

    pub fn get_schema() -> Result<Record, CloudError> {
        DbTools::get_schema::<Self>()
    }

    pub async fn check_table(db: &DatabaseManager) -> Result<(), CloudError> {
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
    pub fn get_id(&self) -> DbInteger {
        self.id.clone()
    }

    pub fn get_player_uuid(&self) -> DbString {
        self.player_uuid.clone()
    }

    pub fn get_service_uuid(&self) -> DbString {
        self.service_uuid.clone()
    }

    // Setter
    pub fn set_player_uuid(&mut self, uuid: DbString) {
        self.player_uuid = uuid;
    }

    pub fn set_service_uuid(&mut self, uuid: DbString) {
        self.service_uuid = uuid;
    }

    // Query Methoden
    pub async fn get_by_session_id(
        db: DatabaseManager,
        id: DbInteger,
    ) -> Result<Option<Self>, CloudError> {
        let mut filter = Record::new();
        filter.insert("id".to_string(), DbValue::Integer(id));

        let records = db.
            get_records(TABLE_PLAYER_SESSIONS, Some(filter))
            .await
            .map_err(|e| error!(CantDBGetRecords, e))?;

        if records.is_empty() {
            return Ok(None);
        }

        Ok(Some(Self::from_record(&records[0])?))
    }

    pub async fn get_by_player_uuid(
        db: &DatabaseManager,
        uuid: DbString,
    ) -> Result<Option<Self>, CloudError> {
        let mut filter = Record::new();
        filter.insert("player_uuid".to_string(), DbValue::String(uuid));

        let records = db
            .get_records(TABLE_PLAYER_SESSIONS, Some(filter))
            .await
            .map_err(|e| error!(CantDBGetRecords, e))?;

        // Nimm nur den ersten Record, wenn vorhanden
        if let Some(first) = records.into_iter().next() {
            Ok(Some(Self::from_record(&first)?))
        } else {
            Ok(None)
        }
    }

}