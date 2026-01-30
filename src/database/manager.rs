use std::error::Error;
use std::sync::Arc;

use crate::database::provider::mysql::DbMysql;
use crate::database::table::table_player_events::TablePlayerEvents;
use crate::database::table::table_player_sessions::TablePlayerSessions;
use crate::database::table::table_players::TablePlayers;
use crate::database::table::table_service_events::TableServiceEvents;
use crate::database::table::table_services::TableServices;
use crate::config::cloud_config::{DBConfig, DBTypes};
use crate::database::db_types::{DbInteger, Record};
use crate::database::provider::DatabaseProvider;
use crate::utils::error::CloudError;

pub struct DatabaseManager {
    db: Arc<dyn DatabaseProvider>
}

impl DatabaseManager {
    pub fn new(
        config: &DBConfig,
    ) -> Result<DatabaseManager, Box<dyn Error + Send + Sync>> {
        let manager = match config.get_type() {
            DBTypes::SQLITE => todo!(), //Arc::new(DbSqlite::new(config.get_sqlite_config())?),
            DBTypes::MYSQL => DatabaseManager {db: Arc::new(DbMysql::new(config.get_mysql_config())?)},
        };

        Ok(manager)
    }

    pub async fn check_tables(&self) -> Result<(), CloudError> {
        // Check Tables
        TableServices::check_table(&self).await?;
        TablePlayers::check_table(&self).await?;
        TablePlayerSessions::check_table(&self).await?;
        TablePlayerEvents::check_table(&self).await?;
        TableServiceEvents::check_table(&self).await?;

        Ok(())
    }


    /// DatabaseProvider 
    pub async fn get_record_from_tables(
        &self,
    ) -> Result<Vec<Record>, Box<dyn Error + Send + Sync>> {
        self.db.get_record_from_tables().await
    }

    pub async fn get_records(
        &self,
        table: &str,
        filter: Option<Record>,
    ) -> Result<Vec<Record>, Box<dyn Error + Send + Sync>> {
        self.db.get_records(table, filter).await
    }

    pub async fn add_record(
        &self,
        table: &str,
        data: Record,
    ) -> Result<DbInteger, Box<dyn Error + Send + Sync>> {
        self.db.add_record(table, data).await
    }

    pub async fn update_record(
        &self,
        table: &str,
        id: DbInteger,
        data: Record,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.db.update_record(table, id, data).await
    }

    pub async fn delete_record(
        &self,
        table: &str,
        id: DbInteger,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.db.delete_record(table, id).await
    }

    pub async fn check_table(
        &self,
        table: &str,
        schema: &Record,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.db.check_table(table, schema).await
    }

}

impl Clone for DatabaseManager {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone()
        }
    }
}
