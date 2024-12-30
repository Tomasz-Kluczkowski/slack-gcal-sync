#[cfg(test)]
mod tests;

use clap::Parser;
use log::{info, warn};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::Path;
use thiserror::Error;

const DEFAULT_SERVICE_ACCOUNT_PATH: &str = ".secrets/.service_account.json";
const DEFAULT_APPLICATION_CONFIG_PATH: &str = "config/application_config.json";
const DEFAULT_CALENDAR_ID: &str = "primary";

#[derive(Debug, Error)]
pub enum ConfigurationError {
    #[error("Cannot read configuration")]
    ReadConfigurationError(#[from] std::io::Error),

    #[error("Cannot deserialize configuration")]
    DeserializeConfigurationError(#[from] serde_json::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationConfiguration {
    pub calendar_id: String,
    pub service_account_key_path: String,
}

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct ApplicationCommandLineArguments {
    #[arg(short, long)]
    pub calendar_id: Option<String>,

    #[arg(short, long)]
    pub service_account_key_path: Option<String>,

    #[arg(short, long, default_value = DEFAULT_APPLICATION_CONFIG_PATH)]
    pub application_config_path: String,
}

pub fn read_json_configuration<T: DeserializeOwned>(config_path: &String) -> Result<T, ConfigurationError> {
    let configuration_file = File::open(config_path)?;
    let configuration: T = serde_json::from_reader(configuration_file)?;
    Ok(configuration)
}

pub fn get_application_configuration(
    args: ApplicationCommandLineArguments,
) -> Result<ApplicationConfiguration, ConfigurationError> {
    let mut app_config: ApplicationConfiguration = if Path::new(args.application_config_path.as_str()).exists() {
        info!(
            "Loading application configuration from file at path: '{}'.",
            args.application_config_path
        );
        let app_config = read_json_configuration::<ApplicationConfiguration>(&args.application_config_path)?;
        info!(
            "Successfully loaded application configuration from file at path: '{}'.",
            args.application_config_path
        );
        app_config
    } else {
        warn!(
            "No application configuration file found at path: '{}'. Using default settings. \
         Make sure to provide configuration via command line args.",
            args.application_config_path
        );
        ApplicationConfiguration {
            service_account_key_path: DEFAULT_SERVICE_ACCOUNT_PATH.to_string(),
            calendar_id: DEFAULT_CALENDAR_ID.to_string(),
        }
    };
    if args.calendar_id.is_some() {
        app_config.calendar_id = args.calendar_id.unwrap().to_string();
    }
    if args.service_account_key_path.is_some() {
        app_config.service_account_key_path = args.service_account_key_path.unwrap().to_string();
    }
    Ok(app_config)
}
