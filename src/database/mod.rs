use database_manager::types::{DBVarChar, Value};
use uuid::Uuid;

use crate::database::table::UUID_LENGTH;

pub mod table;

pub struct DBTools;

impl DBTools {
    pub fn uuid_to_value(uuid: &Uuid) -> Value {
        Value::VarChar(DBVarChar::new(uuid.to_string(), UUID_LENGTH).unwrap())
    }
    pub fn uuid_to_varchar(uuid: &Uuid) -> DBVarChar {
        DBVarChar::new(uuid.to_string(), UUID_LENGTH).unwrap()
    }
}
