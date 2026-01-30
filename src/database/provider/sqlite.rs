use serde::{Deserialize, Serialize};
//use std::path::PathBuf;

//use crate::utils::utils::Utils;

#[derive(Serialize, Deserialize, Clone)]
pub struct DBSqliteConfig {
    file: String,
}

impl DBSqliteConfig {
    /*fn get_path(&self) -> PathBuf {
        Utils::get_path(&self.file)
    }*/
}


