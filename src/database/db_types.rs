use std::collections::HashMap;
use serde::{Deserialize, Serialize};

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
            DbValue::Integer(_) => "BIGINT",
            DbValue::Float(_) => "REAL",
            DbValue::Boolean(_) => "BOOLEAN",
            DbValue::DateTime(_) => "DATETIME",
            DbValue::Date(_) => "DATE",
            DbValue::Null => "TEXT",
        }
    }
}