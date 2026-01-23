use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use crate::database::db_treiber::mysql::DbMysql;
use crate::database::table::table_player_events::TablePlayerEvents;
use crate::database::table::table_player_sessions::TablePlayerSessions;
use crate::database::table::table_players::TablePlayers;
use crate::database::table::table_service_events::TableServiceEvents;
use crate::database::table::table_services::TableServices;
use crate::sys_config::cloud_config::{DBConfig, DBTypes};
use crate::utils::error::CloudError;

#[async_trait]
pub trait DatabaseManager: Send + Sync {
    async fn connect(&mut self) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn disconnect(&mut self) -> Result<(), Box<dyn Error + Send + Sync>>;

    async fn get_record_from_tables(&self) -> Result<Vec<Record>, Box<dyn Error + Send + Sync>>;
    async fn get_records(
        &self,
        table: &str,
        filter: Option<Record>,
    ) -> Result<Vec<Record>, Box<dyn Error + Send + Sync>>;

    async fn add_record(
        &self,
        table: &str,
        data: Record,
    ) -> Result<u64, Box<dyn Error + Send + Sync>>;
    async fn update_record(
        &self,
        table: &str,
        id: u64,
        data: Record,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn delete_record(&self, table: &str, id: u64) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn check_table(&self, table: &str, schema: &Record) -> Result<(), Box<dyn Error + Send + Sync>>;
}
pub type Record = HashMap<String, DbValue>;
pub type DbString = String;
pub type DbDateTime = String;   // format -> yyyy-mm-dd hh-nn-ss <---- MUSS
pub type DbDate = String;       // format -> yyyy-mm-dd         <---- MUSS
pub type DbInteger = i64;
pub type DbFloat = f64;
pub type DbBoolean = bool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DbValue {
    String(DbString),
    Integer(DbInteger),
    Float(DbFloat),
    Boolean(DbBoolean),
    DateTime(DbDateTime),
    Date(DbDate),
    Null,
}

impl DbValue {
    pub fn to_sql_string(&self) -> String {
        match self {
            DbValue::String(s) => format!("'{}'", s.replace('\'', "''")),
            DbValue::Integer(i) => i.to_string(),
            DbValue::Float(f) => f.to_string(),
            DbValue::Boolean(b) => {
                if *b { "1".to_string() } else { "0".to_string() }
            }
            DbValue::DateTime(dt) => {
                // yyyy-mm-dd hh:mm:ss
                format!("'{}'", dt.replace('\'', "''"))
            }
            DbValue::Date(d) => {
                // yyyy-mm-dd
                format!("'{}'", d.replace('\'', "''"))
            }
            DbValue::Null => "NULL".to_string(),
        }
    }

    pub fn to_sql_type_default(&self) -> &str {
        match self {
            DbValue::String(_) => "TEXT",
            DbValue::Integer(_) => "INTEGER",
            DbValue::Float(_) => "REAL",
            DbValue::Boolean(_) => "BOOLEAN",
            DbValue::DateTime(_) => "DATETIME",
            DbValue::Date(_) => "DATE",
            DbValue::Null => "TEXT",
        }
    }
}

pub struct Database;

impl Database {
    pub fn new(
        config: &DBConfig,
    ) -> Result<Arc<dyn DatabaseManager>, Box<dyn Error + Send + Sync>> {
        let manager: Arc<dyn DatabaseManager> = match config.get_type() {
            DBTypes::SQLITE => todo!(), //Arc::new(DbSqlite::new(config.get_sqlite_config())?),
            DBTypes::MYSQL => Arc::new(DbMysql::new(config.get_mysql_config())?),
        };

        Ok(manager)
    }

    pub async fn check_tables(db: Arc<dyn DatabaseManager>) -> Result<(), CloudError> {
        // Check Tables
        TableServices::check_table(&db).await?;
        TablePlayers::check_table(&db).await?;
        TablePlayerSessions::check_table(&db).await?;
        TablePlayerEvents::check_table(&db).await?;
        TableServiceEvents::check_table(&db).await?;

        Ok(())
    }

}
