use configuration::{read_json_configuration, ConfigurationError};
use google_calendar3::yup_oauth2::ServiceAccountKey;

pub fn get_service_account_key(service_account_key_json_path: String) -> Result<ServiceAccountKey, ConfigurationError> {
    Ok(read_json_configuration::<ServiceAccountKey>(
        service_account_key_json_path,
    )?)
}
