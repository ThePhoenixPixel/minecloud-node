use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::core::service::Service;
use crate::database::database_manger::{
    DatabaseManager, DbString, DbValue, Record,
};
use crate::database::db_tools::DbTools;
use crate::error;
use crate::utils::error::CloudError;
use crate::utils::error_kind::CloudErrorKind::*;

const TABLE_NAME: &str = "t_services";
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TableServices {
    service_uuid: DbString,
    service_name: DbString,
    service_type: DbString,
    node: DbString,
    task: DbString,
    status: DbString,
    started_at: DbString, // format -> yyyy-mm-dd hh-nn-ss
    stopped_at: DbString,
}

impl TableServices {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn add(&self, db: Arc<dyn DatabaseManager>) -> Result<(), CloudError> {
        db.add_record(TABLE_NAME, DbTools::struct_to_db_map(self)?)
            .await
            .map_err(|e| error!(CantDBAddRecord, e))?;
        Ok(())
    }

    pub fn get_schema() -> Result<Record, CloudError> {
        DbTools::get_schema::<Self>()
    }

    pub async fn check_table(db: &Arc<dyn DatabaseManager>) -> Result<(), CloudError> {
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

    pub fn setup_from_service(&mut self, service: &Service) {
        self.service_uuid = service.get_id().to_string();
        self.service_name = service.get_name();
        self.service_type = service.get_task().get_software().get_software_type();
        self.task         = service.get_task().get_name();
        self.node         = service.get_start_node();
        //self.started_at   = service.getst
        
    }

    pub fn set_service_uuid(&mut self, service_uuid: DbString) {
        self.service_uuid = service_uuid;
    }

    pub fn set_service_name(&mut self, service_name: DbString) {
        self.service_name = service_name;
    }

    pub fn set_service_type(&mut self, service_type: DbString) {
        self.service_type = service_type;
    }

    pub fn set_started_at(&mut self, started_at: DbString) {
        self.started_at = started_at;
    }

    pub fn set_stopped_at(&mut self, stopped_at: DbString) {
        self.stopped_at = stopped_at;
    }

    pub async fn get_last_service_from_task(
        db: Arc<dyn DatabaseManager>,
        task_name: String,
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
            if let Some(number) = extract_number_from_service_name(&service.service_name, &task_name) {
                if highest_number.is_none() || number > highest_number.unwrap() {
                    highest_number = Some(number);
                    highest_service = Some(service);
                }
            }
        }

        Ok(highest_service)
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

