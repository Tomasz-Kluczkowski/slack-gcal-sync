mod tests;

use clap::Parser;
use log::info;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::Path;
use thiserror::Error;

const DEFAULT_SERVICE_ACCOUNT_PATH: &str = ".secrets/.service_account.json";
const DEFAULT_SLACK_USER_OAUTH_TOKEN_PATH: &str = ".secrets/.slack_user_oauth_token.json";
const DEFAULT_LOGGING_CONFIG_PATH: &str = "config/logging_config.yaml";
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
    pub slack_user_oauth_token_path: String,
    pub logging_config_path: String,
}

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct ApplicationCommandLineArguments {
    #[arg(
        short,
        long,
        help = "The id of the calendar to synchronize. Usually your gmail email account."
    )]
    pub calendar_id: Option<String>,

    #[arg(
        short,
        long,
        help = "Path to the service account key json file from Google Cloud Project."
    )]
    pub service_account_key_path: Option<String>,

    #[arg(short = 't', long, help = "Path to the slack user oauth token json file.")]
    pub slack_user_oauth_token_path: Option<String>,

    #[arg(
        short,
        long,
        help = "Path to the log4rs logging configuration file. See https://docs.rs/log4rs/latest/log4rs/#configuration-via-a-yaml-file for possible options."
    )]
    pub logging_config_path: Option<String>,

    #[arg(short, long, default_value = DEFAULT_APPLICATION_CONFIG_PATH, help = "Path to the application configuration file.")]
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
        info!(
            "No application configuration file found at path: '{}'. Using default settings. \
         Make sure to provide configuration via command line args.",
            args.application_config_path
        );
        ApplicationConfiguration {
            calendar_id: DEFAULT_CALENDAR_ID.to_string(),
            service_account_key_path: DEFAULT_SERVICE_ACCOUNT_PATH.to_string(),
            slack_user_oauth_token_path: DEFAULT_SLACK_USER_OAUTH_TOKEN_PATH.to_string(),
            logging_config_path: DEFAULT_LOGGING_CONFIG_PATH.to_string(),
        }
    };
    if let Some(calendar_id) = args.calendar_id.as_ref() {
        app_config.calendar_id = calendar_id.to_string();
    }
    if let Some(service_account_key_path) = args.service_account_key_path.as_ref() {
        app_config.service_account_key_path = service_account_key_path.to_string();
    }
    if let Some(slack_user_oauth_token_path) = args.slack_user_oauth_token_path.as_ref() {
        app_config.slack_user_oauth_token_path = slack_user_oauth_token_path.to_string();
    }
    if let Some(logging_config_path) = args.logging_config_path.as_ref() {
        app_config.logging_config_path = logging_config_path.to_string();
    }
    Ok(app_config)
}
