mod tests;

use chrono::{Datelike, Duration, TimeZone, Utc};
use google_calendar3::{
    api::Event,
    hyper_rustls,
    hyper_rustls::HttpsConnector,
    hyper_util,
    hyper_util::client::legacy::connect::HttpConnector,
    yup_oauth2::{authenticator::Authenticator, ServiceAccountAuthenticator, ServiceAccountKey},
    CalendarHub, Error as GoogleAPIError,
};
use log::info;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GoogleCalendarIntegrationError {
    #[error("Cannot communicate with Google Calendar API")]
    GoogleCalendarCommunicationError(#[from] Box<GoogleAPIError>),

    #[error("IO Error occurred")]
    IOError(#[from] std::io::Error),
}

impl From<GoogleAPIError> for GoogleCalendarIntegrationError {
    fn from(err: GoogleAPIError) -> Self {
        GoogleCalendarIntegrationError::GoogleCalendarCommunicationError(Box::new(err))
    }
}

pub fn get_calendar_hub(
    authenticator: Authenticator<HttpsConnector<HttpConnector>>,
) -> Result<CalendarHub<HttpsConnector<HttpConnector>>, GoogleCalendarIntegrationError> {
    info!("Setting up google api hub.");
    let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new()).build(
        hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()?
            .https_or_http()
            .enable_http1()
            .build(),
    );
    info!("Successfully set up google api hub.");
    Ok(CalendarHub::new(client, authenticator))
}

pub async fn get_service_account_authenticator(
    service_account_key: ServiceAccountKey,
) -> Result<Authenticator<HttpsConnector<HttpConnector>>, GoogleCalendarIntegrationError> {
    info!("Setting up service account authenticator.");
    let service_account_authenticator = ServiceAccountAuthenticator::builder(service_account_key)
        .build()
        .await?;
    info!("Successfully set up up service account authenticator.");
    Ok(service_account_authenticator)
}

pub async fn get_calendar_events_for_today(
    hub: CalendarHub<HttpsConnector<HttpConnector>>,
    calendar_id: &str,
) -> Result<Vec<Event>, GoogleCalendarIntegrationError> {
    let now = Utc::now();
    let start_of_day = Utc
        .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
        .unwrap();
    let end_of_day = start_of_day + Duration::days(1);
    let (_, events) = hub
        .events()
        .list(calendar_id)
        .time_min(start_of_day)
        .time_max(end_of_day)
        .doit()
        .await?;
    Ok(events.items.unwrap_or_else(Vec::new))
}
