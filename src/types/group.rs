use bx::path::Directory;
use rand::RngExt;
use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{fs, io};

use crate::types::installer::Installer;
use crate::types::template::Template;
use crate::error;
use crate::config::cloud_config::CloudConfig;
use crate::utils::error::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Group {
    name: String,
    installer: Installer,
    templates: Vec<Template>,
}

impl Group {
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_installer(&self) -> Installer {
        self.installer.clone()
    }

    pub fn get_templates(&self) -> Vec<Template> {
        self.templates.clone()
    }

    pub fn get_all() -> Vec<Group> {
        let group_path = CloudConfig::get().get_cloud_path().get_group_folder_path();

        if !group_path.exists() || !group_path.is_dir() {
            return Vec::new();
        }

        fs::read_dir(group_path)
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|entry| {
                let file_str = entry.file_name().to_str()?.to_string();

                file_str.strip_suffix(".json").and_then(Self::get_from_name)
            })
            .collect()
    }

    pub fn get_from_name(name: &str) -> Option<Group> {
        let group_path = CloudConfig::get().get_cloud_path().get_group_folder_path();

        Directory::get_files_name_from_path(&group_path)
            .into_iter()
            .find_map(|file_name| {
                Self::get_from_path(&group_path.join(&file_name))
                    .ok()
                    .filter(|group| group.get_name() == name)
            })
    }

    pub fn get_from_path(path: &PathBuf) -> io::Result<Group> {
        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    pub fn get_templates_sorted_by_priority(&self) -> Vec<Template> {
        let mut templates = self.get_templates();
        templates.sort_by(|a, b| a.priority.cmp(&b.priority));
        templates
    }

    pub fn get_templates_sorted_by_priority_desc(&self) -> Vec<Template> {
        let mut templates = self.get_templates();
        templates.sort_by(|a, b| b.priority.cmp(&a.priority));
        templates
    }

    pub fn get_template_rng(&self) -> Option<&Template> {
        let mut rng = rand::rng();
        self.templates.choose(&mut rng)
    }

    // Select Template based on Priority (higher priority = higher chance)
    pub fn get_template_rng_based_on_priority(&self) -> Option<&Template> {
        if self.templates.is_empty() {
            return None;
        }

        let total_weight: u32 = self.templates.iter().map(|t| t.priority).sum();

        if total_weight == 0 {
            return self.get_template_rng();
        }

        let mut rng = rand::rng();
        let mut random_value = rng.random_range(0..total_weight);

        for template in &self.templates {
            if random_value < template.priority {
                return Some(template);
            }
            random_value -= template.priority;
        }

        // fallback
        self.templates.last()
    }

    pub fn install_in_path(&self, target_path: &PathBuf) -> Result<(), CloudError> {
        let mut templates: Vec<Template> = Vec::new();
        match self.get_installer() {
            Installer::InstallAll => templates = self.get_templates_sorted_by_priority(),
            Installer::InstallAllDesc => templates = self.get_templates_sorted_by_priority_desc(),
            Installer::InstallRandom => match self.get_template_rng() {
                Some(template) => templates.push(template.clone()),
                None => return Err(error!(GroupTemplateNotFound)),
            },
            Installer::InstallRandomWithPriority => {
                match self.get_template_rng_based_on_priority() {
                    Some(template) => templates.push(template.clone()),
                    None => return Err(error!(GroupTemplateNotFound)),
                }
            }
        }

        for template in templates {
            Directory::copy_folder_contents(&template.get_path(), &target_path)
                .map_err(|e| error!(CantCopyGroupTemplateToNewServiceFolder, e))?;
        }
        Ok(())
    }
}
