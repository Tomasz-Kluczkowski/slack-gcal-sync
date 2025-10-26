use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::rolling_file::policy::compound;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::config::{load_config_file, Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::{Config, Handle};
use thiserror::Error;

pub const DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::Info;
pub const DEFAULT_LOG_FILE_PATH: &str = "logs";
pub const DEFAULT_LOG_FILE_BASE_NAME: &str = "slack-gcal-sync";
pub const DEFAULT_LOG_FILE_EXTENSION: &str = "log";
pub const DEFAULT_CONSOLE_LOG_PATTERN: &str = "{d(%+)(utc)} [{f}:{L}] {h({l})} {M}:{m}{n}";
pub const DEFAULT_ROLLING_FILE_BASE_INDEX: u32 = 0;
pub const DEFAULT_ROLLING_FILE_ARCHIVE_COUNT: u32 = 5;
pub const DEFAULT_LOG_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10 MB size limit
                                                         // pub const DEFAULT_LOG_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10 MB size limit
pub const DEFAULT_CONSOLE_LOGGER_NAME: &str = "stdout";
pub const DEFAULT_ROLLING_FILE_LOGGER_NAME: &str = "rollingfile";

#[derive(Debug, Error)]
pub enum LoggerError {
    #[error("Cannot read logger configuration at path: {0}. {1}")]
    ReadLoggerConfigurationError(String, String),
}

pub struct LoggerConfigurator {
    pub log_file_path: Option<String>,
    pub log_file_roller_pattern: Option<String>,
    pub log_file_size: Option<u64>,
}

impl Default for LoggerConfigurator {
    fn default() -> Self {
        LoggerConfigurator {
            log_file_path: Some(Self::get_default_log_file_path()),
            log_file_roller_pattern: Some(Self::get_default_log_file_roller_pattern()),
            log_file_size: Some(DEFAULT_LOG_FILE_SIZE),
        }
    }
}

impl LoggerConfigurator {
    pub fn get_default_log_file_roller_pattern() -> String {
        format!("{DEFAULT_LOG_FILE_PATH}/{DEFAULT_LOG_FILE_BASE_NAME}-{{}}.{DEFAULT_LOG_FILE_EXTENSION}")
    }

    pub fn get_default_log_file_path() -> String {
        format!("{DEFAULT_LOG_FILE_PATH}/{DEFAULT_LOG_FILE_BASE_NAME}.{DEFAULT_LOG_FILE_EXTENSION}")
    }

    pub fn setup_default_logger(&self) -> Handle {
        let stdout = ConsoleAppender::builder()
            .encoder(Box::new(PatternEncoder::new(DEFAULT_CONSOLE_LOG_PATTERN)))
            .build();
        let roller = FixedWindowRoller::builder()
            .base(DEFAULT_ROLLING_FILE_BASE_INDEX)
            .build(
                self.log_file_roller_pattern.clone().unwrap().as_str(),
                DEFAULT_ROLLING_FILE_ARCHIVE_COUNT,
            )
            .unwrap();
        let trigger = SizeTrigger::new(self.log_file_size.unwrap());
        let policy = compound::CompoundPolicy::new(Box::new(trigger), Box::new(roller));
        let rollingfile = RollingFileAppender::builder()
            .encoder(Box::new(log4rs::encode::json::JsonEncoder::new()))
            .build(self.log_file_path.clone().unwrap().as_str(), Box::new(policy))
            .unwrap();
        let config = Config::builder()
            .appender(Appender::builder().build(DEFAULT_CONSOLE_LOGGER_NAME, Box::new(stdout)))
            .appender(Appender::builder().build(DEFAULT_ROLLING_FILE_LOGGER_NAME, Box::new(rollingfile)))
            .build(
                Root::builder()
                    .appender(DEFAULT_CONSOLE_LOGGER_NAME)
                    .appender(DEFAULT_ROLLING_FILE_LOGGER_NAME)
                    .build(DEFAULT_LOG_LEVEL),
            )
            .unwrap();

        log4rs::init_config(config).unwrap()
    }

    pub fn apply_logging_config_from_file(
        &self,
        logging_config_path: &str,
        logging_handle: &Handle,
    ) -> Result<(), LoggerError> {
        match load_config_file(logging_config_path, Default::default()) {
            Ok(config_from_file) => {
                logging_handle.set_config(config_from_file);
                Ok(())
            }
            Err(err) => Err(LoggerError::ReadLoggerConfigurationError(
                logging_config_path.to_string(),
                err.to_string(),
            )),
        }
    }
}
