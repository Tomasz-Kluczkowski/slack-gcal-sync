use google_calendar3::hyper_rustls::HttpsConnector;
use google_calendar3::hyper_util::client::legacy::connect::HttpConnector;
use google_calendar3::yup_oauth2::{ServiceAccountAuthenticator, ServiceAccountKey};
use google_calendar3::{hyper_rustls, hyper_util, CalendarHub, Error as GoogleAPIError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GoogleCalendarIntegrationError {
    #[error("Cannot communicate with Google Calendar Calendar API")]
    GoogleCalendarCommunicationError(#[from] GoogleAPIError),

    #[error("IO Error occurred")]
    IOError(#[from] std::io::Error),
}

pub async fn get_calendar_hub(
    service_account_key: ServiceAccountKey,
) -> Result<CalendarHub<HttpsConnector<HttpConnector>>, GoogleCalendarIntegrationError> {
    let service_account_authenticator = ServiceAccountAuthenticator::builder(service_account_key)
        .build()
        .await?;

    let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new()).build(
        hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()?
            .https_or_http()
            .enable_http1()
            .build(),
    );
    Ok(CalendarHub::new(client, service_account_authenticator))
}
