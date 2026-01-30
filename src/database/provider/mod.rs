use std::error::Error;
use async_trait::async_trait;
use crate::database::db_types::{DbInteger, Record};

pub mod mysql;
pub mod sqlite;


#[async_trait]
pub trait DatabaseProvider: Send + Sync {
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
    ) -> Result<DbInteger, Box<dyn Error + Send + Sync>>;
    async fn update_record(
        &self,
        table: &str,
        id: DbInteger,
        data: Record,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn delete_record(&self, table: &str, id: DbInteger) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn check_table(&self, table: &str, schema: &Record) -> Result<(), Box<dyn Error + Send + Sync>>;
}

