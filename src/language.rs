use colored::{ColoredString, Colorize};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::cloud::Cloud;
use crate::config::CloudConfig;

pub struct Language;

#[derive(Debug, Serialize, Deserialize)]
struct LanguageData {
    version_1: HashMap<String, String>,
}

impl Language {
    pub fn translate(key: &str) -> ColoredString {
        // Deserialisiere die JSON-Daten in die Struktur
        let lang_file_path = PathBuf::from(format!(
            "{:?}{}",
            Cloud::get_working_path(),
            CloudConfig::get().get_language()
        ));
        let file_content =
            fs::read_to_string(lang_file_path).unwrap_or_else(|_| Language::get_default_content());

        //file content in die langstrukt pressen :))
        let language_data: LanguageData = match serde_json::from_str(file_content.as_str()) {
            Ok(content) => content,
            Err(_) => {
                return ColoredString::from(format!("Error translate key = {} ", key).as_str())
                    .red();
            }
        };

        // Überprüfe, ob der Schlüssel im HashMap vorhanden ist not retrun the para key
        return match language_data.version_1.get(key).map(String::as_str) {
            Some(content) => ColoredString::from(content),
            None => ColoredString::from(format!("Error translate key = {} ", key).as_str()).red(),
        };
    }

    fn get_default_content() -> String {
        return String::from(
            r#"
        {
            "version_1": {
                "start": "Start GameCloud ...",
                "shutdown": "Gob Bye"
            }
        }
    "#,
        );
    }
}
