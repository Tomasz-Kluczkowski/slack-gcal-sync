mod tests;

use chrono::{Datelike, Duration, TimeZone, Utc};
use google_calendar3::api::Event;
use google_calendar3::hyper_rustls::HttpsConnector;
use google_calendar3::hyper_util::client::legacy::connect::HttpConnector;
use google_calendar3::yup_oauth2::authenticator::Authenticator;
use google_calendar3::{hyper_rustls, hyper_util, CalendarHub, Error as GoogleAPIError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GoogleCalendarIntegrationError {
    #[error("Cannot communicate with Google Calendar API")]
    GoogleCalendarCommunicationError(#[from] GoogleAPIError),

    #[error("IO Error occurred")]
    IOError(#[from] std::io::Error),
}

pub async fn get_calendar_hub(
    authenticator: Authenticator<HttpsConnector<HttpConnector>>,
) -> Result<CalendarHub<HttpsConnector<HttpConnector>>, GoogleCalendarIntegrationError> {
    let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new()).build(
        hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()?
            .https_or_http()
            .enable_http1()
            .build(),
    );
    Ok(CalendarHub::new(client, authenticator))
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
