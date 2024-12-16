use clap::Parser;
use serde::de::DeserializeOwned;
use std::fs::File;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigurationError {
    #[error("Cannot read configuration")]
    ReadConfigurationError(#[from] std::io::Error),

    #[error("Cannot deserialize configuration")]
    DeserializeConfigurationError(#[from] serde_json::Error),
}

pub fn read_json_configuration<T: DeserializeOwned>(config_path: &String) -> Result<T, ConfigurationError> {
    let configuration_file = File::open(config_path)?;
    let configuration: T = serde_json::from_reader(configuration_file)?;
    Ok(configuration)
}

const DEFAULT_SERVICE_ACCOUNT_PATH: &str = ".service_account/.service_account.json";
const DEFAULT_APPLICATION_CONFIG_PATH: &str = "config/application_config.json";

pub struct ApplicationConfiguration {
    pub service_account_key_path: String,
    pub calendar_id: String,
}

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct ApplicationCommandLineArguments {
    #[arg(short, long)]
    pub calendar_id: String,

    #[arg(short, long, default_value = DEFAULT_SERVICE_ACCOUNT_PATH)]
    pub service_account_key_path: String,

    #[arg(short, long, default_value = DEFAULT_APPLICATION_CONFIG_PATH)]
    pub application_config_path: String,
}

#[cfg(test)]
mod tests {
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
