use bx::network::address::Address;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use crate::cloud::Cloud;
use crate::log_error;

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

    pub fn replace_placeholders(
        strings: Vec<String>,
        replacements: &HashMap<&str, String>,
    ) -> Vec<String> {
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

    pub fn get_path(s: &String) -> PathBuf {
        //check ob relativ '~'
        // or windows C:/..
        // linux zb /home or /opt
        let mut path = PathBuf::new();

        if s.find('~').is_some() {
            path.push(Cloud::get_working_path());
        }
        path.push(s.trim_matches('~'));
        path
    }

    pub fn get_datetime_now() -> String {
        let now: DateTime<Utc> = Utc::now();
        now.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    /// Optional: Nur das Datum "YYYY-MM-DD"
    pub fn get_date_now() -> String {
        let now: DateTime<Utc> = Utc::now();
        now.format("%Y-%m-%d").to_string()
    }

    pub async fn wait_nano(nano: u128) {
        tokio::time::sleep(Duration::from_nanos(nano as u64)).await;
    }

    pub async fn wait_sec(sec: u64) {
        tokio::time::sleep(Duration::from_secs(sec)).await;
    }

    pub fn find_free_port(used_ports: &[u32], mut port: u32, host: &String) -> u32 {
        while used_ports.contains(&port) || !Address::is_port_available(&Address::new(host, &port))
        {
            port += 1;
        }
        port
    }
}
