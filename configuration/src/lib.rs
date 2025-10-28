mod tests;

use std::{fs::File, path::Path};

use clap::Parser;
use google_calendar3::yup_oauth2::ServiceAccountKey;
use log::info;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
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

    #[error("Cannot deserialize configuration from path: {0}. {1}")]
    DeserializeConfigurationError(String, String),

    #[error(
        "Missing value(s) for application configuration: {0}. Check application configuration file and command line arguments/environment variables."
    )]
    InvalidConfigurationError(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationConfiguration {
    pub calendar_id: String,
    pub service_account_key: ServiceAccountKey,
    pub slack_user_oauth_token: String,
    pub logging_config_path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SlackUserOauthToken {
    user_oauth_token: String,
}

#[derive(Parser, Debug, Serialize, Deserialize, Clone, PartialEq)]
#[command(version, about)]
pub struct ApplicationConfigurationData {
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

    #[arg(env, help = "Environment variable holding value of slack user oauth token.")]
    pub slack_user_oauth_token: Option<String>,

    #[arg(
        short,
        long,
        help = "Path to the log4rs logging configuration file. See https://docs.rs/log4rs/latest/log4rs/#configuration-via-a-yaml-file for possible options."
    )]
    pub logging_config_path: Option<String>,

    #[arg(short, long, default_value = DEFAULT_APPLICATION_CONFIG_PATH, help = "Path to the application configuration file.")]
    pub application_config_path: Option<String>,
}

impl Default for ApplicationConfigurationData {
    fn default() -> Self {
        ApplicationConfigurationData {
            calendar_id: Some(DEFAULT_CALENDAR_ID.to_string()),
            service_account_key_path: Some(DEFAULT_SERVICE_ACCOUNT_PATH.to_string()),
            slack_user_oauth_token_path: Some(DEFAULT_SLACK_USER_OAUTH_TOKEN_PATH.to_string()),
            slack_user_oauth_token: None,
            logging_config_path: Some(DEFAULT_LOGGING_CONFIG_PATH.to_string()),
            application_config_path: Some(DEFAULT_APPLICATION_CONFIG_PATH.to_string()),
        }
    }
}

impl ApplicationConfigurationData {
    fn update(&mut self, other: ApplicationConfigurationData) {
        if let Some(calendar_id) = other.calendar_id.as_ref() {
            self.calendar_id = Some(calendar_id.to_string());
        }
        if let Some(service_account_key_path) = other.service_account_key_path.as_ref() {
            self.service_account_key_path = Some(service_account_key_path.to_string());
        }
        if let Some(slack_user_oauth_token_path) = other.slack_user_oauth_token_path.as_ref() {
            self.slack_user_oauth_token_path = Some(slack_user_oauth_token_path.to_string());
        }
        if let Some(slack_user_oauth_token) = other.slack_user_oauth_token.as_ref() {
            self.slack_user_oauth_token = Some(slack_user_oauth_token.to_string());
        }
        if let Some(logging_config_path) = other.logging_config_path.as_ref() {
            self.logging_config_path = Some(logging_config_path.to_string());
        }
        if let Some(application_config_path) = other.application_config_path.as_ref() {
            self.application_config_path = Some(application_config_path.to_string());
        }
    }
}

pub struct ApplicationConfigurationGetter {
    cli_application_configuration_data: ApplicationConfigurationData,
    file_application_configuration_data: ApplicationConfigurationData,
}

impl ApplicationConfigurationGetter {
    pub fn new(cli_application_configuration_data: ApplicationConfigurationData) -> Result<Self, ConfigurationError> {
        let file_application_configuration_data = Self::load_from_file_or_default(
            cli_application_configuration_data
                .application_config_path
                .clone()
                .unwrap_or(DEFAULT_APPLICATION_CONFIG_PATH.to_string())
                .as_str(),
        )?;
        Ok(ApplicationConfigurationGetter {
            cli_application_configuration_data,
            file_application_configuration_data,
        })
    }

    fn load_from_file_or_default(
        application_config_path: &str,
    ) -> Result<ApplicationConfigurationData, ConfigurationError> {
        let file_application_configuration_data: ApplicationConfigurationData =
            if Path::new(application_config_path).exists() {
                info!(
                    "Loading application configuration from file at path: '{}'.",
                    application_config_path
                );
                let file_application_configuration_data =
                    read_json_configuration::<ApplicationConfigurationData>(&application_config_path.to_string())?;
                info!(
                    "Successfully loaded application configuration from file at path: '{}'.",
                    application_config_path
                );
                file_application_configuration_data
            } else {
                info!(
                    "No application configuration file found at path: '{}'. Using default settings. \
         Make sure to provide configuration via command line args.",
                    application_config_path
                );
                ApplicationConfigurationData::default()
            };
        Ok(file_application_configuration_data)
    }

    fn get_service_account_key(&self, service_account_key_path: &str) -> Result<ServiceAccountKey, ConfigurationError> {
        info!(
            "Reading google calendar service account key from path: '{}'.",
            service_account_key_path
        );

        match read_json_configuration::<ServiceAccountKey>(&service_account_key_path.to_string()) {
            Ok(service_account_key) => {
                info!("Successfully read google calendar service account key.");
                Ok(service_account_key)
            }
            Err(err) => Err(ConfigurationError::DeserializeConfigurationError(
                service_account_key_path.to_string(),
                err.to_string(),
            )),
        }
    }

    fn get_slack_user_oauth_token(
        &self,
        slack_user_oauth_token_path: &str,
    ) -> Result<SlackUserOauthToken, ConfigurationError> {
        info!(
            "Reading slack user oauth token from path: '{}'.",
            slack_user_oauth_token_path
        );

        match read_json_configuration::<SlackUserOauthToken>(&slack_user_oauth_token_path.to_string()) {
            Ok(slack_user_oauth_token) => {
                info!("Successfully read slack user oauth token.");
                Ok(slack_user_oauth_token)
            }
            Err(err) => Err(ConfigurationError::DeserializeConfigurationError(
                slack_user_oauth_token_path.to_string(),
                err.to_string(),
            )),
        }
    }

    fn validate(
        &self,
        application_configuration_data: &ApplicationConfigurationData,
    ) -> Result<(), ConfigurationError> {
        let mut missing_fields = Vec::new();

        if application_configuration_data.calendar_id.is_none() {
            missing_fields.push("calendar_id is required".to_string());
        }
        if application_configuration_data.service_account_key_path.is_none() {
            missing_fields.push("service_account_key_path is required".to_string());
        }
        if application_configuration_data.slack_user_oauth_token_path.is_none()
            && application_configuration_data.slack_user_oauth_token.is_none()
        {
            missing_fields.push("Either slack_user_oauth_token_path or slack_user_oauth_token must be set".to_string());
        }
        if application_configuration_data.logging_config_path.is_none() {
            missing_fields.push("logging_config_path is required".to_string());
        }
        // We set the default value in cli args to always contain application_config_path. Therefore, currently
        // validation for application_config_path should always pass. It is kept in validation though to provide a guard
        // for the future if logic changes or default is removed.
        if application_configuration_data.application_config_path.is_none() {
            missing_fields.push("application_config_path is required".to_string());
        }

        if missing_fields.is_empty() {
            Ok(())
        } else {
            Err(ConfigurationError::InvalidConfigurationError(missing_fields.join(", ")))
        }
    }

    pub fn get_application_configuration(&self) -> Result<ApplicationConfiguration, ConfigurationError> {
        let mut merged_application_configuration_data = self.file_application_configuration_data.clone();
        merged_application_configuration_data.update(self.cli_application_configuration_data.clone());
        self.validate(&merged_application_configuration_data.clone())?;
        let service_account_key_path = merged_application_configuration_data.service_account_key_path.clone();

        let service_account_key = self.get_service_account_key(service_account_key_path.as_ref().unwrap())?;

        let slack_user_oauth_token = merged_application_configuration_data
            .slack_user_oauth_token
            .clone()
            .unwrap_or(
                self.get_slack_user_oauth_token(
                    merged_application_configuration_data
                        .slack_user_oauth_token_path
                        .as_ref()
                        .unwrap(),
                )?
                .user_oauth_token,
            );

        Ok(ApplicationConfiguration {
            calendar_id: merged_application_configuration_data.calendar_id.clone().unwrap(),
            service_account_key,
            slack_user_oauth_token,
            logging_config_path: merged_application_configuration_data
                .logging_config_path
                .clone()
                .unwrap(),
        })
    }
}

pub fn read_json_configuration<T: DeserializeOwned>(config_path: &String) -> Result<T, ConfigurationError> {
    let configuration_file = File::open(config_path)?;
    match serde_json::from_reader(configuration_file) {
        Ok(configuration) => Ok(configuration),
        Err(e) => Err(ConfigurationError::DeserializeConfigurationError(
            config_path.to_string(),
            e.to_string(),
        )),
    }
}
