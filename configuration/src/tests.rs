#[cfg(test)]
mod test_application_configuration_getter {
    use std::io::Write;

    use google_calendar3::yup_oauth2::ServiceAccountKey;
    use rstest::{fixture, rstest};
    use tempfile::NamedTempFile;

    use crate::{
        ApplicationConfigurationData, ApplicationConfigurationGetter, ConfigurationError, SlackUserOauthToken,
    };

    #[fixture]
    fn file_app_config_data() -> ApplicationConfigurationData {
        ApplicationConfigurationData {
            calendar_id: Some("calendar@gmail.com".to_string()),
            service_account_key_path: Some("path/.service_account_key.json".to_string()),
            slack_user_oauth_token_path: Some("path/.slack_user_oauth_token.json".to_string()),
            slack_user_oauth_token: None,
            logging_config_path: Some("path/logging_config.yaml".to_string()),
            application_config_path: None,
        }
    }

    #[fixture]
    fn service_account_key() -> ServiceAccountKey {
        ServiceAccountKey {
            key_type: Some("service_account".to_string()),
            project_id: Some("project_id".to_string()),
            private_key_id: Some("private_key_id".to_string()),
            private_key: "-----BEGIN PRIVATE KEY-----\nMIIE".to_string(),
            client_email: "test@test.com".to_string(),
            client_id: Some("client_id".to_string()),
            auth_uri: Some("https://accounts.google.com/o/oauth2/auth".to_string()),
            token_uri: "https://oauth2.googleapis.com/token".to_string(),
            auth_provider_x509_cert_url: Some("https://www.googleapis.com/oauth2/v1/certs".to_string()),
            client_x509_cert_url: Some(
                "https://www.googleapis.com/robot/v1/metadata/x509/example.gserviceaccount.com".to_string(),
            ),
        }
    }

    #[fixture]
    fn slack_user_oauth_token() -> SlackUserOauthToken {
        SlackUserOauthToken {
            user_oauth_token: "redacted".to_string(),
        }
    }

    fn write_json_to_temp_file<T: serde::Serialize>(data: &T) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        let json_data = serde_json::to_string_pretty(data).unwrap();
        file.write_all(json_data.as_bytes()).unwrap();
        file
    }

    #[rstest]
    fn it_should_create_new_instance_using_default_application_configuration_data_when_no_configuration_file_found() {
        let cli_app_config_data = ApplicationConfigurationData {
            calendar_id: None,
            service_account_key_path: None,
            slack_user_oauth_token_path: None,
            slack_user_oauth_token: None,
            logging_config_path: None,
            application_config_path: None,
        };

        let application_configuration_getter =
            ApplicationConfigurationGetter::new(cli_app_config_data.clone()).unwrap();

        assert_eq!(
            application_configuration_getter.cli_application_configuration_data,
            cli_app_config_data
        );
        assert_eq!(
            application_configuration_getter.file_application_configuration_data,
            ApplicationConfigurationData::default()
        );
    }

    #[rstest]
    fn test_get_application_configuration_using_config_file_path(
        mut file_app_config_data: ApplicationConfigurationData,
        service_account_key: ServiceAccountKey,
        slack_user_oauth_token: SlackUserOauthToken,
    ) {
        let service_account_key_file = write_json_to_temp_file(&service_account_key);
        let service_account_key_path = service_account_key_file.path().to_str().unwrap().to_string();

        let slack_user_oauth_token_file = write_json_to_temp_file(&slack_user_oauth_token);
        let slack_user_oauth_token_path = slack_user_oauth_token_file.path().to_str().unwrap().to_string();

        file_app_config_data.service_account_key_path = Some(service_account_key_path);
        file_app_config_data.slack_user_oauth_token_path = Some(slack_user_oauth_token_path);
        let application_config_file = write_json_to_temp_file(&file_app_config_data);
        let app_config_path = application_config_file.path().to_str().unwrap().to_string();

        let cli_app_config_data = ApplicationConfigurationData {
            calendar_id: None,
            service_account_key_path: None,
            slack_user_oauth_token_path: None,
            slack_user_oauth_token: None,
            logging_config_path: None,
            application_config_path: Some(app_config_path),
        };

        let calendar_id = file_app_config_data.calendar_id.clone().unwrap();
        let logging_config_path = file_app_config_data.logging_config_path.clone().unwrap();
        let application_configuration_getter = ApplicationConfigurationGetter::new(cli_app_config_data).unwrap();

        let output_configuration = application_configuration_getter
            .get_application_configuration()
            .unwrap();

        assert_eq!(output_configuration.calendar_id, calendar_id);

        assert_eq!(
            output_configuration.service_account_key.key_type,
            service_account_key.key_type
        );
        assert_eq!(
            output_configuration.service_account_key.project_id,
            service_account_key.project_id
        );
        assert_eq!(
            output_configuration.service_account_key.private_key_id,
            service_account_key.private_key_id
        );
        assert_eq!(
            output_configuration.service_account_key.private_key,
            service_account_key.private_key
        );
        assert_eq!(
            output_configuration.service_account_key.client_email,
            service_account_key.client_email
        );
        assert_eq!(
            output_configuration.service_account_key.client_id,
            service_account_key.client_id
        );
        assert_eq!(
            output_configuration.service_account_key.auth_uri,
            service_account_key.auth_uri
        );
        assert_eq!(
            output_configuration.service_account_key.token_uri,
            service_account_key.token_uri
        );
        assert_eq!(
            output_configuration.service_account_key.auth_provider_x509_cert_url,
            service_account_key.auth_provider_x509_cert_url
        );
        assert_eq!(
            output_configuration.service_account_key.client_x509_cert_url,
            service_account_key.client_x509_cert_url
        );

        assert_eq!(
            output_configuration.slack_user_oauth_token,
            slack_user_oauth_token.user_oauth_token
        );
        assert_eq!(output_configuration.logging_config_path, logging_config_path);
    }

    #[rstest]
    fn test_get_application_configuration_uses_cli_arguments_to_override_file_based_configuration(
        file_app_config_data: ApplicationConfigurationData,
        service_account_key: ServiceAccountKey,
        slack_user_oauth_token: SlackUserOauthToken,
    ) {
        let service_account_key_file = write_json_to_temp_file(&service_account_key);
        let service_account_key_path = service_account_key_file.path().to_str().unwrap().to_string();

        let slack_user_oauth_token_file = write_json_to_temp_file(&slack_user_oauth_token);
        let slack_user_oauth_token_path = slack_user_oauth_token_file.path().to_str().unwrap().to_string();

        let application_config_file = write_json_to_temp_file(&file_app_config_data);
        let app_config_path = application_config_file.path().to_str().unwrap().to_string();

        let slack_user_oauth_token_value = "fake_token";
        let cli_app_config_data = ApplicationConfigurationData {
            calendar_id: Some("overridden_calendar_id".to_string()),
            service_account_key_path: Some(service_account_key_path),
            slack_user_oauth_token_path: Some(slack_user_oauth_token_path),
            slack_user_oauth_token: Some(slack_user_oauth_token_value.to_string()),
            logging_config_path: Some("overridden_logging_config_path".to_string()),
            application_config_path: Some(app_config_path),
        };

        let calendar_id = cli_app_config_data.calendar_id.clone().unwrap();
        let logging_config_path = cli_app_config_data.logging_config_path.clone().unwrap();
        let application_configuration_getter = ApplicationConfigurationGetter::new(cli_app_config_data).unwrap();

        let output_configuration = application_configuration_getter
            .get_application_configuration()
            .unwrap();

        assert_eq!(output_configuration.calendar_id, calendar_id);

        assert_eq!(
            output_configuration.service_account_key.key_type,
            service_account_key.key_type
        );
        assert_eq!(
            output_configuration.service_account_key.project_id,
            service_account_key.project_id
        );
        assert_eq!(
            output_configuration.service_account_key.private_key_id,
            service_account_key.private_key_id
        );
        assert_eq!(
            output_configuration.service_account_key.private_key,
            service_account_key.private_key
        );
        assert_eq!(
            output_configuration.service_account_key.client_email,
            service_account_key.client_email
        );
        assert_eq!(
            output_configuration.service_account_key.client_id,
            service_account_key.client_id
        );
        assert_eq!(
            output_configuration.service_account_key.auth_uri,
            service_account_key.auth_uri
        );
        assert_eq!(
            output_configuration.service_account_key.token_uri,
            service_account_key.token_uri
        );
        assert_eq!(
            output_configuration.service_account_key.auth_provider_x509_cert_url,
            service_account_key.auth_provider_x509_cert_url
        );
        assert_eq!(
            output_configuration.service_account_key.client_x509_cert_url,
            service_account_key.client_x509_cert_url
        );

        assert_eq!(
            output_configuration.slack_user_oauth_token,
            slack_user_oauth_token_value
        );
        assert_eq!(output_configuration.logging_config_path, logging_config_path);
    }

    #[test]
    fn test_get_application_configuration_validates_merged_configuration_and_returns_missing_arguments() {
        // Currently, an application config path is always specified in default cli arguments if missing, so it is not
        // possible to fail validation for it. To avoid using a default application config path, we specify it in cli args.
        let file_app_config_data = ApplicationConfigurationData {
            calendar_id: None,
            service_account_key_path: None,
            slack_user_oauth_token_path: None,
            slack_user_oauth_token: None,
            logging_config_path: None,
            application_config_path: None,
        };

        let application_config_file = write_json_to_temp_file(&file_app_config_data);
        let app_config_path = application_config_file.path().to_str().unwrap().to_string();

        let cli_app_config_data = ApplicationConfigurationData {
            calendar_id: None,
            service_account_key_path: None,
            slack_user_oauth_token_path: None,
            slack_user_oauth_token: None,
            logging_config_path: None,
            application_config_path: Some(app_config_path),
        };

        let application_configuration_getter = ApplicationConfigurationGetter::new(cli_app_config_data).unwrap();

        let invalid_result = application_configuration_getter.get_application_configuration();
        assert!(matches!(
            invalid_result,
            Err(ConfigurationError::InvalidConfigurationError { .. })
        ));
        let error_contents = invalid_result.unwrap_err().to_string();
        assert!(error_contents.contains("calendar_id is required"));
        assert!(error_contents.contains("service_account_key_path is required"));
        assert!(error_contents.contains("Either slack_user_oauth_token_path or slack_user_oauth_token must be set"));
        assert!(error_contents.contains("logging_config_path is required"))
    }

    #[rstest]
    fn test_get_application_configuration_reports_invalid_service_account_key(
        mut file_app_config_data: ApplicationConfigurationData,
    ) {
        let service_account_key_path = "invalid_path/service_account_key.json";
        file_app_config_data.service_account_key_path = Some(service_account_key_path.to_string());

        let application_config_file = write_json_to_temp_file(&file_app_config_data);
        let app_config_path = application_config_file.path().to_str().unwrap().to_string();

        let cli_app_config_data = ApplicationConfigurationData {
            calendar_id: None,
            service_account_key_path: None,
            slack_user_oauth_token_path: None,
            slack_user_oauth_token: None,
            logging_config_path: None,
            application_config_path: Some(app_config_path),
        };

        let application_configuration_getter = ApplicationConfigurationGetter::new(cli_app_config_data).unwrap();

        let invalid_result = application_configuration_getter.get_application_configuration();
        assert!(matches!(
            invalid_result,
            Err(ConfigurationError::DeserializeConfigurationError { .. })
        ));
        let error_contents = invalid_result.unwrap_err().to_string();
        assert!(error_contents.contains(service_account_key_path));
        assert!(error_contents.contains("Cannot deserialize configuration from path:"));
        assert!(error_contents.contains("Cannot read configuration"));
    }

    #[rstest]
    fn test_get_application_configuration_reports_invalid_slack_user_oath_token_file(
        mut file_app_config_data: ApplicationConfigurationData,
        service_account_key: ServiceAccountKey,
    ) {
        let service_account_key_file = write_json_to_temp_file(&service_account_key);
        let service_account_key_path = service_account_key_file.path().to_str().unwrap().to_string();
        file_app_config_data.service_account_key_path = Some(service_account_key_path);

        let slack_user_oauth_token_path = "invalid_path/slack_user_oauth_token.json";
        file_app_config_data.slack_user_oauth_token_path = Some(slack_user_oauth_token_path.to_string());

        let application_config_file = write_json_to_temp_file(&file_app_config_data);
        let app_config_path = application_config_file.path().to_str().unwrap().to_string();

        let cli_app_config_data = ApplicationConfigurationData {
            calendar_id: None,
            service_account_key_path: None,
            slack_user_oauth_token_path: None,
            slack_user_oauth_token: None,
            logging_config_path: None,
            application_config_path: Some(app_config_path),
        };

        let application_configuration_getter = ApplicationConfigurationGetter::new(cli_app_config_data).unwrap();

        let invalid_result = application_configuration_getter.get_application_configuration();
        assert!(matches!(
            invalid_result,
            Err(ConfigurationError::DeserializeConfigurationError { .. })
        ));
        let error_contents = invalid_result.unwrap_err().to_string();
        assert!(error_contents.contains(slack_user_oauth_token_path));
        assert!(error_contents.contains("Cannot deserialize configuration from path:"));
        assert!(error_contents.contains("Cannot read configuration"));
    }
}

#[cfg(test)]
mod test_read_json_configuration {
    use std::io::Write;

    use serde::{Deserialize, Serialize};
    use tempfile::NamedTempFile;

    use crate::{read_json_configuration, ConfigurationError};

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
