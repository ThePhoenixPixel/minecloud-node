use serde::{Deserialize, Serialize};

use crate::types::service::Service;
use crate::database::manager::DatabaseManager;
use crate::database::db_tools::DbTools;
use crate::database::db_types::*;
use crate::error;
use crate::utils::error::CloudError;
use crate::utils::error_kind::CloudErrorKind::*;
use crate::utils::service_status::ServiceStatus;

const TABLE_NAME: &str = "t_services";
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TableServices {
    id: DbInteger,                  // service ID
    created_at: DbDateTime,         // format -> YYYY-MM-DD HH:MM:SS

    uuid: DbString,
    name: DbString,
    typ: DbString,
    node: DbString,
    task: DbString,
    status: DbString,
    host: DbString,
    started_at: DbDateTime,         // format -> YYYY-MM-DD HH:MM:SS
    stopped_at: DbDateTime,         // format -> YYYY-MM-DD HH:MM:SS
}

impl TableServices {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_from_service(service: &Service) -> Self {
        let mut db_service= Self::new();
        db_service.uuid           = service.get_id().to_string();
        db_service.name           = service.get_name();
        db_service.typ            = service.get_task().get_software().get_software_type();
        db_service.task           = service.get_task().get_name();
        db_service.node           = service.get_start_node();
        db_service.status         = service.get_status().to_string();
        db_service.host           = service.get_server_listener().to_string();
        db_service.started_at     = DbString::new();
        db_service.stopped_at     = DbString::new();
        db_service
    }

    pub async fn add(&self, db: &DatabaseManager) -> Result<(), CloudError> {
        db.add_record(TABLE_NAME, DbTools::struct_to_db_map(self)?)
            .await
            .map_err(|e| error!(CantCreateDBRecord, e))?;
        Ok(())
    }

    pub async fn update(&self, db: &DatabaseManager) -> Result<(), CloudError> {
        db.update_record(TABLE_NAME, self.id, DbTools::struct_to_db_map(&self)?)
            .await
            .map_err(|e| error!(CantUpdateDBRecord, e))
    }

    pub fn get_schema() -> Result<Record, CloudError> {
        DbTools::get_schema::<Self>()
    }

    pub async fn check_table(db: &DatabaseManager) -> Result<(), CloudError> {
        db.check_table(TABLE_NAME, &Self::get_schema()?)
            .await
            .map_err(|e| error!(CantCreateTable, e))?;
        Ok(())
    }

    fn from_record(record: &Record) -> Result<Self, CloudError> {
        let json = DbTools::record_to_json(record);
        serde_json::from_value(json)
            .map_err(|e| error!(DeserializationError, e))
    }

    fn from_records(records: Vec<Record>) -> Result<Vec<TableServices>, CloudError> {
        let mut result: Vec<TableServices> = Vec::new();
        for rec in records {
            let typ = Self::from_record(&rec)?;
            result.push(typ);
        }
        Ok(result)
    }

    pub fn get_uuid(&self) -> DbString {
        self.uuid.clone()
    }

    pub fn get_name(&self) -> DbString {
        self.name.clone()
    }

    pub fn get_typ(&self) -> DbString {
        self.typ.clone()
    }

    pub fn get_node(&self) -> DbString {
        self.node.clone()
    }

    pub fn get_task(&self) -> DbString {
        self.task.clone()
    }

    pub fn get_status(&self) -> ServiceStatus {
        ServiceStatus::from_string(&self.status)
    }

    pub fn get_started_at(&self) -> DbDateTime {
        self.started_at.clone()
    }

    pub fn get_stopped_at(&self) -> DbDateTime {
        self.stopped_at.clone()
    }


    // ===== Setter =====
    pub fn set_uuid(&mut self, uuid: DbString) {
        self.uuid = uuid;
    }

    pub fn set_name(&mut self, name: DbString) {
        self.name = name;
    }

    pub fn set_typ(&mut self, r#type: DbString) {
        self.typ = r#type;
    }

    pub fn set_node(&mut self, node: DbString) {
        self.node = node;
    }

    pub fn set_task(&mut self, task: DbString) {
        self.task = task;
    }

    pub fn set_status(&mut self, status: &ServiceStatus) {
        self.status = status.to_string();
    }

    pub fn set_started_at(&mut self, started_at: DbDateTime) {
        self.started_at = started_at;
    }

    pub fn set_stopped_at(&mut self, stopped_at: DbDateTime) {
        self.stopped_at = stopped_at;
    }

    pub async fn get_last_service_from_task(
        db: &DatabaseManager,
        task_name: DbString,
    ) -> Result<Option<Self>, CloudError> {
        let mut filter: Record = Record::new();
        filter.insert("task".to_string(), DbValue::String(task_name.clone()));

        let records = db
            .get_records(TABLE_NAME, Some(filter))
            .await
            .map_err(|e| error!(CantDBGetRecords, e))?;

        if records.is_empty() {
            return Ok(None);
        }

        let services = Self::from_records(records)?;

        let mut highest_number: Option<u64> = None;
        let mut highest_service: Option<Self> = None;

        for service in services {
            if let Some(number) = extract_number_from_service_name(&service.name, &task_name) {
                if highest_number.is_none() || number > highest_number.unwrap() {
                    highest_number = Some(number);
                    highest_service = Some(service);
                }
            }
        }

        Ok(highest_service)
    }

    pub async fn get_from_uuid(
        db: &DatabaseManager,
        uuid: DbString,
    ) -> Result<Option<TableServices>, CloudError> {
        let mut filter: Record = Record::new();
        filter.insert("uuid".to_string(), DbValue::String(uuid));

        let records = db
            .get_records(TABLE_NAME, Some(filter))
            .await
            .map_err(|e| error!(CantDBGetRecords, e))?;

        if records.is_empty() {
            return Ok(None);
        }

        for service in Self::from_records(records)? {
            return Ok(Some(service));
        }

        Ok(None)
    }

}

fn extract_number_from_service_name(service_name: &str, task_name: &str) -> Option<u64> {
    // Entferne den task_name vom Anfang
    if let Some(rest) = service_name.strip_prefix(task_name) {
        // Überspringe das Split-Zeichen (erstes Zeichen nach task_name)
        if rest.len() > 1 {
            let number_str = &rest[1..]; // Überspringe das Split-Zeichen
            return number_str.parse::<u64>().ok();
        }
    }
    None
}

