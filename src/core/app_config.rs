use crate::model::config::{Config, ConfigTable};
use crate::model::error::system::SystemError;
use crate::model::error::Error;
use crate::model::log::system::SystemLog;
use macros::log;
use std::fs;
use std::ops::Deref;

pub struct AppConfig {
    config: Config,
}

impl AppConfig {
    pub fn new() -> Result<Self, Error> {
        log!(SystemLog::Initializing);
        let config = Self::load_config_file()?;
        log!(SystemLog::InitializeComplete);
        Ok(Self { config })
    }

    fn load_config_file() -> Result<Config, Error> {
        let toml_string =
            fs::read_to_string("./config.toml").map_err(SystemError::ConfigNotFound)?;
        let config = toml::from_str::<ConfigTable>(&toml_string)
            .map_err(SystemError::InvalidConfig)?
            .config;
        Ok(config)
    }
}

impl Deref for AppConfig {
    type Target = Config;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}
