use serde::de::DeserializeOwned;
use std::fs::File;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigurationError {
    #[error("Cannot read configuration on path: '{path}'. {message}")]
    ReadConfigurationError { path: String, message: String },

    #[error("Cannot deserialize configuration: {0}")]
    DeserializeConfigurationError(#[from] serde_json::Error),
}

pub fn read_json_configuration<T: DeserializeOwned>(config_path: String) -> Result<T, ConfigurationError> {
    match File::open(&config_path) {
        Ok(configuration_file) => {
            let configuration: T = serde_json::from_reader(configuration_file)?;
            Ok(configuration)
        }
        Err(error) => Err(ConfigurationError::ReadConfigurationError {
            path: config_path,
            message: error.to_string(),
        }),
    }
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

        let output_struct = read_json_configuration::<TestPerson>(file.path().display().to_string()).unwrap();
        assert_eq!(output_struct, input_struct);
        file.close().unwrap();
    }

    #[test]
    fn it_should_raise_read_configuration_error_on_file_missing() {
        let nonexistent_file_path = "nonexistent_dir/nonexistent_file.json";
        let expected_error_message = format!(
            "Cannot read configuration on path: '{file_path}'. No such file or directory (os error 2)",
            file_path = nonexistent_file_path
        );
        let error = read_json_configuration::<TestPerson>(nonexistent_file_path.to_string());

        assert!(matches!(error, Err(ConfigurationError::ReadConfigurationError { .. })));
        assert_eq!(error.unwrap_err().to_string(), expected_error_message);
    }

    #[test]
    fn it_should_raise_deserialize_configuration_error_on_invalid_json() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"###invalid_json_file!!!").unwrap();

        let error = read_json_configuration::<TestPerson>(file.path().display().to_string());
        assert!(matches!(
            error,
            Err(ConfigurationError::DeserializeConfigurationError { .. })
        ));
        assert!(error
            .unwrap_err()
            .to_string()
            .contains("Cannot deserialize configuration:"));
        file.close().unwrap();
    }
}
