use std::collections::HashMap;
use serde::Serialize;
use serde_json::Value;

use crate::database::db_types::{DbValue, Record};
use crate::error;
use crate::utils::error::CloudError;
use crate::utils::error_kind::CloudErrorKind::*;

pub struct DbTools;

impl DbTools {
    pub fn record_to_json(record: &Record) -> Value {
        let mut map = serde_json::Map::new();

        for (key, value) in record {
            let json_val = match value {
                DbValue::String(s) => Value::String(s.clone()),
                DbValue::Boolean(b) => Value::Bool(*b),
                DbValue::Integer(i) => Value::Number((*i).into()),
                DbValue::Float(f) => {
                    serde_json::Number::from_f64(*f)
                        .map(Value::Number)
                        .unwrap_or(Value::Null)
                }
                DbValue::DateTime(dt) => Value::String(dt.clone()),
                DbValue::Date(d) => Value::String(d.clone()),
                DbValue::Null => Value::Null,
            };

            map.insert(key.clone(), json_val);
        }

        Value::Object(map)
    }

    pub fn struct_to_db_map<T: Serialize>(input: &T) -> Result<HashMap<String, DbValue>, CloudError> {
        let value = serde_json::to_value(input)
            .map_err(|e| error!(CantParseToValue, e))?;

        let obj = value.as_object().ok_or(error!(CantParseToValue))?;

        let mut map = HashMap::new();

        for (key, val) in obj {
            let db_val = match val {
                Value::String(s) => {
                    if is_datetime(s) {
                        DbValue::DateTime(s.clone())
                    } else if is_date(s) {
                        DbValue::Date(s.clone())
                    } else {
                        DbValue::String(s.clone())
                    }
                }
                Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        DbValue::Integer(i)
                    } else if let Some(f) = n.as_f64() {
                        DbValue::Float(f)
                    } else {
                        DbValue::Null
                    }
                }
                Value::Bool(b) => DbValue::Boolean(*b),
                Value::Null => DbValue::Null,
                _ => DbValue::Null,
            };

            map.insert(key.clone(), db_val);
        }

        Ok(map)
    }

    pub fn get_schema<T>() -> Result<Record, CloudError>
    where
        T: serde::Serialize + Default,
    {
        let dummy = T::default();

        let json = serde_json::to_value(&dummy)
            .map_err(|e| error!(CantGetSchema, e))?;

        let mut schema = Record::new();

        if let Value::Object(map) = json {
            for (key, value) in map {
                let db_value = match value {
                    Value::String(s) => {
                        if is_datetime(&s) {
                            DbValue::DateTime(String::new())
                        } else if is_date(&s) {
                            DbValue::Date(String::new())
                        } else {
                            DbValue::String(String::new())
                        }
                    }
                    Value::Number(n) if n.is_i64() => DbValue::Integer(0),
                    Value::Number(_) => DbValue::Float(0.0),
                    Value::Bool(_) => DbValue::Boolean(false),
                    _ => DbValue::String(String::new()),
                };

                schema.insert(key, db_value);
            }
        }

        Ok(schema)
    }

}

fn is_datetime(s: &str) -> bool {
    // yyyy-mm-dd hh:mm:ss
    s.len() == 19 && s.chars().nth(10) == Some(' ')
}

fn is_date(s: &str) -> bool {
    // yyyy-mm-dd
    s.len() == 10 && s.chars().nth(4) == Some('-')
}
