use std::collections::HashMap;
use async_trait::async_trait;
use bx::network::address::Address;
use mysql;
use mysql::prelude::Queryable;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::database::database_manger::{DatabaseManager, DbInteger, DbValue, Record};

#[derive(Serialize, Deserialize, Clone)]
pub struct DBMysqlConfig {
    host: Address,
    username: String,
    password: String,
    database: String,
    pool_size: u32,
}

impl DBMysqlConfig {
    fn get_url(&self) -> String {
        format!(
            "mysql://{username}:{pw}@{host}/{db}",
            username = self.username,
            pw = self.password,
            host = self.host.to_string(),
            db = self.database
        )
    }
}

pub struct DbMysql {
    pool: Arc<RwLock<mysql::Pool>>,
}

impl DbMysql {
    pub fn new(config: DBMysqlConfig) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let pool = mysql::Pool::new(config.get_url().as_str())?;
        Ok(Self {
            pool: Arc::new(RwLock::new(pool)),
        })
    }

    fn table_exists(&self, conn: &mut mysql::PooledConn, table: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let query = "SELECT COUNT(*) as count FROM information_schema.tables \
                     WHERE table_schema = DATABASE() AND table_name = ?";

        let result: Option<u64> = conn.exec_first(query, (table,))?;
        Ok(result.unwrap_or(0) > 0)
    }

    fn get_existing_columns(
        &self,
        conn: &mut mysql::PooledConn,
        table: &str,
    ) -> Result<HashMap<String, String>, Box<dyn Error + Send + Sync>> {
        let query = "SELECT COLUMN_NAME, DATA_TYPE \
                     FROM information_schema.columns \
                     WHERE table_schema = DATABASE() AND table_name = ?";

        let rows: Vec<(String, String)> = conn.exec(query, (table,))?;

        let mut columns = HashMap::new();
        for (name, data_type) in rows {
            columns.insert(name, data_type);
        }

        Ok(columns)
    }

    fn db_value_to_mysql_type(value: &DbValue) -> &str {
        match value {
            DbValue::String(_) => "TEXT",
            DbValue::Boolean(_) => "BOOLEAN",
            DbValue::Integer(_) => "INT",
            DbValue::Float(_) => "DOUBLE",
            DbValue::DateTime(_) => "DATETIME",
            DbValue::Date(_) => "DATE",
            DbValue::Null => "VARCHAR(1)",
        }
    }

    fn build_create_table_sql(table: &str, schema: &Record) -> String {
        let mut columns = Vec::new();

        // PK id zuerst
        columns.push("id INT PRIMARY KEY AUTO_INCREMENT".to_string());

        // Rest der Spalten, id überspringen
        for (name, value) in schema {
            if name != "id" {
                let sql_type = Self::db_value_to_mysql_type(value);
                columns.push(format!("{} {}", name, sql_type));
            }
        }

        format!("CREATE TABLE IF NOT EXISTS {} ({})", table, columns.join(", "))
    }

    fn build_alter_table_sql(
        table: &str,
        schema: &Record,
        existing_columns: &HashMap<String, String>,
    ) -> Vec<String> {
        let mut alters = Vec::new();

        for (name, value) in schema {
            // id überspringen, wenn sie schon in der Struct ist
            if name != "id" && !existing_columns.contains_key(name) {
                let sql_type = Self::db_value_to_mysql_type(value);
                alters.push(format!("ALTER TABLE {} ADD COLUMN {} {}", table, name, sql_type));
            }
        }

        alters
    }

}

#[async_trait]
impl DatabaseManager for DbMysql {
    async fn connect(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }

    async fn get_record_from_tables(&self) -> Result<Vec<Record>, Box<dyn Error + Send + Sync>> {
        todo!()
    }

    async fn get_records(
        &self,
        table: &str,
        filter: Option<Record>,
    ) -> Result<Vec<Record>, Box<dyn Error + Send + Sync>> {
        let mut sql = format!("SELECT * FROM {}", table);

        // Filter umsetzen
        if let Some(f) = filter {
            let conditions: Vec<String> = f
                .iter()
                .map(|(k, v)| format!("{} = {}", k, v.to_sql_string()))
                .collect();
            sql.push_str(&format!(" WHERE {}", conditions.join(" AND ")));
        }

        // Verbindung holen
        let pool = self.pool.read().await;
        let mut conn = pool.get_conn()?;

        // Query ausführen
        let rows: Vec<mysql::Row> = conn.query(sql)?;

        let mut records = Vec::new();

        for row in rows {
            let mut record = Record::new();
            let columns = row.columns_ref();

            for (idx, col) in columns.iter().enumerate() {
                let col_name = col.name_str().to_string();

                // Spaltenwert als mysql::Value
                let value = row
                    .get_opt::<mysql::Value, _>(idx)
                    .unwrap_or(Ok(mysql::Value::NULL))
                    .unwrap_or(mysql::Value::NULL);

                // Wert in DbValue umwandeln
                let db_value = match value {
                    mysql::Value::NULL => DbValue::Null,
                    mysql::Value::Bytes(b) => {
                        // Prüfen, ob es eine Zahl ist
                        let s = String::from_utf8_lossy(&b).to_string();
                        if let Ok(i) = s.parse::<i64>() {
                            DbValue::Integer(i)
                        } else if let Ok(f) = s.parse::<f64>() {
                            DbValue::Float(f)
                        } else {
                            DbValue::String(s)
                        }
                    }
                    mysql::Value::Int(i) => DbValue::Integer(i),
                    mysql::Value::UInt(u) => DbValue::Integer(u as i64),
                    mysql::Value::Float(f) => DbValue::Float(f.into()),
                    mysql::Value::Double(d) => DbValue::Float(d),
                    _ => DbValue::Null,
                };

                record.insert(col_name, db_value);
            }

            records.push(record);
        }

        // Falls Tabelle leer → Dummy Record mit id
        if records.is_empty() {
            let mut schema = Record::new();
            schema.insert("id".to_string(), DbValue::Integer(0));
            records.push(schema);
        }

        Ok(records)
    }



    async fn add_record(
        &self,
        table: &str,
        data: Record,
    ) -> Result<DbInteger, Box<dyn Error + Send + Sync>> {
        let columns: Vec<String> = data.keys().cloned().collect();
        let values: Vec<String> = data.values().map(|v| v.to_sql_string()).collect();
        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            table,
            columns.join(", "),
            values.join(", ")
        );

        let pool = self.pool.read().await;
        let mut conn = pool.get_conn()?;
        conn.query_drop(&sql)?;

        let id: Option<DbInteger> = conn.query_first("SELECT LAST_INSERT_ID()")?;
        Ok(id.unwrap_or(0))
    }

    async fn update_record(
        &self,
        table: &str,
        id: DbInteger,
        mut data: Record,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        data.remove("id");

        if data.is_empty() {
            return Ok(());
        }

        let updates: Vec<String> = data
            .iter()
            .map(|(k, v)| format!("{} = {}", k, v.to_sql_string()))
            .collect();

        let sql = format!(
            "UPDATE {} SET {} WHERE id = {}",
            table,
            updates.join(", "),
            id
        );

        let pool = self.pool.read().await;
        let mut conn = pool.get_conn()?;

        conn.query_drop(&sql)?;

        Ok(())
    }

    async fn delete_record(
        &self,
        table: &str,
        id: DbInteger,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let sql = format!("DELETE FROM {} WHERE id = {}", table, id);

        let pool = self.pool.read().await;
        let mut conn = pool.get_conn()?;
        conn.query_drop(&sql)?;

        Ok(())
    }

    async fn check_table(
        &self,
        table: &str,
        schema: &Record,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let pool = self.pool.write().await;
        let mut conn = pool.get_conn()?;

        let table_exists = self.table_exists(&mut conn, table)?;

        if !table_exists {
            let create_sql = Self::build_create_table_sql(table, schema);
            conn.query_drop(&create_sql)?;
        } else {
            let existing_columns = self.get_existing_columns(&mut conn, table)?;
            let alter_statements = Self::build_alter_table_sql(table, schema, &existing_columns);

            for sql in alter_statements {
                conn.query_drop(&sql)?;
            }
        }

        Ok(())
    }
}