#[cfg(test)]
mod test_logging {

    use crate::DEFAULT_LOG_FILE_BASE_NAME;
    use crate::DEFAULT_LOG_FILE_EXTENSION;
    use crate::LoggerConfigurator;
    use crate::{DEFAULT_LOG_FILE_PATH, DEFAULT_LOG_FILE_SIZE, LoggerError};
    use log::{error, info, warn};
    use std::fs::{File, create_dir_all, read_to_string, remove_dir_all};
    use std::io::{Seek, SeekFrom, Write};
    use tempfile::NamedTempFile;

    fn write_yaml_to_temp_file(yaml_string: &str) -> NamedTempFile {
        let mut file = NamedTempFile::with_suffix(".yaml".to_string()).unwrap();
        file.write_all(yaml_string.as_bytes()).unwrap();
        file
    }

    #[test]
    fn test_get_default_log_file_roller_pattern() {
        let default_log_file_roller_pattern = LoggerConfigurator::get_default_log_file_roller_pattern();
        assert_eq!(
            default_log_file_roller_pattern,
            format!("{DEFAULT_LOG_FILE_PATH}/{DEFAULT_LOG_FILE_BASE_NAME}-{{}}.{DEFAULT_LOG_FILE_EXTENSION}")
        )
    }

    #[test]
    fn test_get_default_log_file_path() {
        let default_log_file_path = LoggerConfigurator::get_default_log_file_path();
        assert_eq!(
            default_log_file_path,
            format!("{DEFAULT_LOG_FILE_PATH}/{DEFAULT_LOG_FILE_BASE_NAME}.{DEFAULT_LOG_FILE_EXTENSION}")
        )
    }

    #[test]
    fn test_default_creates_default_logger_configurator() {
        let logger_configurator = LoggerConfigurator::default();
        assert_eq!(
            logger_configurator.log_file_path.unwrap(),
            LoggerConfigurator::get_default_log_file_path()
        );
        assert_eq!(
            logger_configurator.log_file_roller_pattern.unwrap(),
            LoggerConfigurator::get_default_log_file_roller_pattern()
        );
        assert_eq!(logger_configurator.log_file_size.unwrap(), DEFAULT_LOG_FILE_SIZE);
    }

    // #[test]
    // fn test_setup_default_logger() {
    //     let test_logs_root_path = "/tmp/test_logs";
    //     create_dir_all(test_logs_root_path.to_string()).unwrap();
    //     let log_file_path = format!("{test_logs_root_path}/test.log");
    //     let log_file_roller_pattern = format!("{}/test-log-{{}}.log", test_logs_root_path);
    //     let rolled_log_file_path = log_file_roller_pattern.replace("{}", "0");
    //
    //     let log_file_size: u64 = 1024;
    //
    //     let mut existing_large_log_file = File::create(log_file_path.to_string()).unwrap();
    //     existing_large_log_file.set_len(log_file_size).unwrap();
    //     existing_large_log_file.seek(SeekFrom::Start(0)).unwrap();
    //     let existing_log_file_content = vec![b'A'; log_file_size as usize];
    //     existing_large_log_file.write_all(&existing_log_file_content).unwrap();
    //
    //     let logger_configurator = LoggerConfigurator {
    //         log_file_path: Some(log_file_path),
    //         log_file_roller_pattern: Some(log_file_roller_pattern),
    //         log_file_size: Some(log_file_size), // Deliberately small size to trigger rolling pattern.
    //     };
    //     let _logging_handle = logger_configurator.setup_default_logger();
    //     let info_log = "This is an info log message.";
    //     let warn_log = "This is a warn log message.";
    //     let error_log = "This is an error log message.";
    //
    //     info!("{info_log}");
    //     warn!("{warn_log}");
    //     error!("{error_log}");
    //
    //     let newest_logs = read_to_string(logger_configurator.log_file_path.clone().unwrap()).unwrap();
    //
    //     // After overflowing, new logs still go to the main log file.
    //     assert!(newest_logs.contains(warn_log));
    //     assert!(newest_logs.contains(error_log));
    //
    //     // Old logs are in rolling pattern log file. since file size is already large only the first
    //     // log will appear in the rolled (old) log file.
    //     let rolled_logs = read_to_string(rolled_log_file_path).unwrap();
    //     assert!(rolled_logs.contains(info_log));
    //     remove_dir_all(test_logs_root_path.to_string()).unwrap();
    // }

    //     #[test]
    //     fn test_apply_logging_config_from_file_modifies_default_logger_configuration() {
    //         let test_logs_root_path = "/tmp/file_based_config_logs";
    //         create_dir_all(test_logs_root_path.to_string()).unwrap();
    //         let log_file_path = format!("{test_logs_root_path}/test.log");
    //         let log_file_roller_pattern = format!("{}/test-log-{{}}.log", test_logs_root_path);
    //         let rolled_log_file_path = log_file_roller_pattern.replace("{}", "0");
    //
    //         let log_file_size: u64 = 1024;
    //
    //         let mut existing_large_log_file = File::create(log_file_path.to_string()).unwrap();
    //         existing_large_log_file.set_len(log_file_size).unwrap();
    //         existing_large_log_file.seek(SeekFrom::Start(0)).unwrap();
    //         let existing_log_file_content = vec![b'A'; log_file_size as usize];
    //         existing_large_log_file.write_all(&existing_log_file_content).unwrap();
    //
    //         let yaml_string = format!(
    //             r#"
    // refresh_rate: 120 seconds
    //
    // appenders:
    //     stdout:
    //         kind: console
    //         encoder:
    //             pattern: "{{d(%+)(utc)}} [{{f}}:{{L}}] {{h({{l}})}} {{M}}:{{m}}{{n}}"
    //         filters:
    //             - kind: threshold
    //               level: info
    //     rollingfile:
    //         kind: rolling_file
    //         path: "{log_file_path}"
    //         encoder:
    //             kind: json
    //         policy:
    //             kind: compound
    //             trigger:
    //                 kind: size
    //                 limit: {log_file_size}
    //             roller:
    //                 kind: fixed_window
    //                 pattern: "{log_file_roller_pattern}"
    //                 base: 0
    //                 count: 5
    // root:
    //     level: info
    //     appenders:
    //         - stdout
    //         - rollingfile
    // "#
    //         );
    //
    //         let logging_config_file = write_yaml_to_temp_file(&yaml_string);
    //         let logging_config_file_path = logging_config_file.path().to_str().unwrap().to_string();
    //
    //         let logger_configurator = LoggerConfigurator::default();
    //         let logging_handle = logger_configurator.setup_default_logger();
    //         let _logging_handle = logger_configurator
    //             .apply_logging_config_from_file(&logging_config_file_path, &logging_handle)
    //             .unwrap();
    //
    //         let info_log = "This is an info log message.";
    //         let warn_log = "This is a warn log message.";
    //         let error_log = "This is an error log message.";
    //
    //         info!("{info_log}");
    //         warn!("{warn_log}");
    //         error!("{error_log}");
    //
    //         let newest_logs = read_to_string(logger_configurator.log_file_path.clone().unwrap()).unwrap();
    //
    //         // After overflowing, new logs still go to the main log file.
    //         assert!(newest_logs.contains(warn_log));
    //         assert!(newest_logs.contains(error_log));
    //
    //         // Old logs are in rolling pattern log file. since file size is already large only the first
    //         // log will appear in the rolled (old) log file.
    //         let rolled_logs = read_to_string(rolled_log_file_path).unwrap();
    //         assert!(rolled_logs.contains(info_log));
    //         remove_dir_all(test_logs_root_path.to_string()).unwrap();
    //     }

    #[test]
    fn test_apply_logging_config_from_file_error_handling() {
        let logger_configurator = LoggerConfigurator::default();
        let logging_handle = logger_configurator.setup_default_logger();

        let non_existent_config_path = "/tmp/non_existent_config.yaml";
        let invalid_result =
            logger_configurator.apply_logging_config_from_file(non_existent_config_path, &logging_handle);
        assert!(matches!(
            invalid_result,
            Err(LoggerError::ReadLoggerConfigurationError { .. })
        ));
        let error_contents = invalid_result.unwrap_err().to_string();

        assert!(
            error_contents.contains(
                format!(
                    "Cannot read logger configuration at path: {}. No such file or directory (os error 2)",
                    non_existent_config_path
                )
                .as_str()
            )
        );
    }
}
