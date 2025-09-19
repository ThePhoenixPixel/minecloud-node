use std::collections::HashMap;
use crate::log_error;
use crate::utils::logger::Logger;
use serde::Serialize;

pub struct Utils;

impl Utils {
    pub fn convert_to_json<T>(value: &T) -> Option<serde_json::Value>
    where
        T: ?Sized + Serialize,
    {
        let json_string = match serde_json::to_string_pretty(value) {
            Ok(json_string) => json_string,
            Err(e) => {
                log_error!("{}", e.to_string());
                return None;
            }
        };

        match serde_json::from_str(json_string.as_str()) {
            Ok(json) => Some(json),
            Err(e) => {
                log_error!("{}", e.to_string());
                None
            }
        }
    }

    pub fn replace_placeholders(strings: Vec<String>, replacements: &HashMap<&str, String>) -> Vec<String> {
        strings
            .into_iter()
            .map(|s| {
                let mut result = s.clone();
                for (placeholder, value) in replacements {
                    let placeholder_with_percent = format!("%{}%", placeholder);
                    result = result.replace(&placeholder_with_percent, value);
                }
                result
            })
            .collect()
    }
    
}
