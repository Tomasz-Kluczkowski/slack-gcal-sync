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
            "Loading application configuration from file at path: {}.",
            args.application_config_path
        );
        read_json_configuration::<ApplicationConfiguration>(&args.application_config_path)?
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

#[cfg(test)]
mod test_get_application_configuration {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn it_should_read_application_configuration_file() {
        // application config path specified in command line args,
        // specified app config file should be used,
        // resulting config should match what is in json file.
        let mut file = NamedTempFile::new().unwrap();
        let app_config_path = file.path().to_str().unwrap().to_string();
        let args = ApplicationCommandLineArguments {
            calendar_id: None,
            service_account_key_path: None,
            application_config_path: app_config_path,
        };

        let app_config_file_data = ApplicationConfiguration {
            calendar_id: "calendar@gmail.com".to_string(),
            service_account_key_path: "path/service_account_key.json".to_string(),
        };
        let json = serde_json::to_string_pretty(&app_config_file_data).unwrap();
        file.write_all(&json.as_bytes()).unwrap();

        let output_app_config = get_application_configuration(args).unwrap();
        assert_eq!(output_app_config.calendar_id, app_config_file_data.calendar_id);
        assert_eq!(
            output_app_config.service_account_key_path,
            app_config_file_data.service_account_key_path
        );
    }

    #[test]
    fn it_should_use_default_application_configuration() {
        // no command line args at all, specified app config file does not exist.
        // default config values should be used.
        let args = ApplicationCommandLineArguments {
            calendar_id: None,
            service_account_key_path: None,
            application_config_path: "non_existent_app_config.json".to_string(),
        };

        let app_config = get_application_configuration(args).unwrap();
        assert_eq!(app_config.calendar_id, DEFAULT_CALENDAR_ID.to_string());
        assert_eq!(
            app_config.service_account_key_path,
            DEFAULT_SERVICE_ACCOUNT_PATH.to_string()
        );
    }

    #[test]
    fn it_should_use_command_line_args() {
        // all app config command line args specified,
        // specified app config file should be used, but values should be overridden by cli args.
        // resulting config should match cli args.
        let mut file = NamedTempFile::new().unwrap();
        let app_config_path = file.path().to_str().unwrap().to_string();
        let calendar_id_arg = "overridden@gmail.com".to_string();
        let service_account_key_path_arg = "overridden_service_account_key.json".to_string();
        let args = ApplicationCommandLineArguments {
            calendar_id: Some(calendar_id_arg.clone()),
            service_account_key_path: Some(service_account_key_path_arg.clone()),
            application_config_path: app_config_path,
        };

        let app_config_file_data = ApplicationConfiguration {
            calendar_id: "calendar@gmail.com".to_string(),
            service_account_key_path: "path/service_account_key.json".to_string(),
        };
        let json = serde_json::to_string_pretty(&app_config_file_data).unwrap();
        file.write_all(&json.as_bytes()).unwrap();

        let output_app_config = get_application_configuration(args).unwrap();
        assert_eq!(output_app_config.calendar_id, calendar_id_arg);
        assert_eq!(output_app_config.service_account_key_path, service_account_key_path_arg);
    }
}

#[cfg(test)]
mod test_read_json_configuration {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestPerson {
        name: String,
        age: i8,
    }

    #[test]
    fn it_should_read_json_configuration_and_create_struct() {
        let input_struct = TestPerson {
            name: "tom".to_string(),
            age: 123,
        };
        let json = serde_json::to_string_pretty(&input_struct).unwrap();

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(&json.as_bytes()).unwrap();

        let output_struct = read_json_configuration::<TestPerson>(&file.path().display().to_string()).unwrap();
        assert_eq!(output_struct, input_struct);
        file.close().unwrap();
    }

    #[test]
    fn it_should_raise_read_configuration_error_on_file_missing() {
        let nonexistent_file_path = "nonexistent_dir/nonexistent_file.json";
        let expected_error_message = "Cannot read configuration".to_string();
        let error = read_json_configuration::<TestPerson>(&nonexistent_file_path.to_string());

        assert!(matches!(error, Err(ConfigurationError::ReadConfigurationError { .. })));
        assert_eq!(error.unwrap_err().to_string(), expected_error_message);
    }

    #[test]
    fn it_should_raise_deserialize_configuration_error_on_invalid_json() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"###invalid_json_file!!!").unwrap();

        let error = read_json_configuration::<TestPerson>(&file.path().display().to_string());
        assert!(matches!(
            error,
            Err(ConfigurationError::DeserializeConfigurationError { .. })
        ));
        assert!(error
            .unwrap_err()
            .to_string()
            .contains("Cannot deserialize configuration"));
        file.close().unwrap();
    }
}
