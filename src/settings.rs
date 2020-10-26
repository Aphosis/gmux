use super::{Error, Result};
use config::Config;
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub store: PathBuf,
    pub executable: PathBuf,
    pub current: Option<String>,
}

impl Settings {
    pub fn load() -> Result<Self> {
        let mut settings = Config::default();
        settings
            .merge(config::File::with_name("Settings.toml"))
            .unwrap()
            .merge(config::Environment::with_prefix("RIT"))
            .unwrap();

        if let Some(path) = Settings::user_config_file() {
            if path.is_file() {
                let name = path.to_str().unwrap();
                settings.merge(config::File::with_name(name)).unwrap();
            }
        }

        let settings = settings.try_into::<Settings>().unwrap();

        settings.ensure_store()?;

        Ok(settings)
    }

    pub fn save(&self) -> Result<()> {
        match Settings::user_config_file() {
            Some(path) => {
                let mut writer = File::create(path)?;
                let raw = toml::to_string_pretty(self)?;
                write!(writer, "{}", raw)?;
                Ok(())
            }
            None => Err(Error::InvalidSettingsFile.into()),
        }
    }

    pub fn store_full_path(&self) -> PathBuf {
        if self.store.is_absolute() {
            return self.store.clone();
        } else if let Some(path) = Settings::app_config_dir() {
            return path.join(&self.store);
        }
        self.store.clone()
    }

    fn ensure_store(&self) -> Result<()> {
        create_dir_all(self.store_full_path())?;
        Ok(())
    }

    fn app_config_dir() -> Option<PathBuf> {
        match config_dir() {
            Some(path) => Some(path.join("rit")),
            None => None,
        }
    }

    fn user_config_file() -> Option<PathBuf> {
        if let Some(path) = Settings::app_config_dir() {
            let user_settings_path = path.join("rit.toml");
            // Ensure this is a valid OS path.
            if let Some(_) = user_settings_path.to_str() {
                return Some(user_settings_path);
            }
        }
        None
    }
}
