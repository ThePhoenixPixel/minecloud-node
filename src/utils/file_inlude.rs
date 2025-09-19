use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Include {
    local: bool,
    extract: bool,
    url: String,
    path: String,
}

impl Include {
    pub fn get_local(&self) -> bool {
        self.local
    }

    pub fn get_extract(&self) -> bool {
        self.extract
    }

    pub fn get_url(&self) -> &str {
        &self.url
    }

    pub fn get_path(&self) -> &str {
        &self.path
    }

    pub fn set_local(&mut self, value: bool) {
        self.local = value;
    }

    pub fn set_extract(&mut self, value: bool) {
        self.extract = value;
    }

    pub fn set_url<S: Into<String>>(&mut self, value: S) {
        self.url = value.into();
    }

    pub fn set_path<S: Into<String>>(&mut self, value: S) {
        self.path = value.into();
    }
}
