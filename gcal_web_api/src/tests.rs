#[cfg(test)]
mod test_utils {
    use mockito::{Mock, ServerGuard};

    pub fn setup_default_crypto_provider() {
        let _ = rustls::crypto::ring::default_provider().install_default();
    }

    pub async fn get_mock_auth_setup() -> (Mock, ServerGuard) {
        let mut mock_auth_server = mockito::Server::new_async().await;

        let mock_auth_api_response = serde_json::json!({
            "authorization": "Bearer blah"
        });

        let mock_auth_api = mock_auth_server
            .mock("POST", mockito::Matcher::Any)
            .with_body(mock_auth_api_response.to_string())
            .with_status(200)
            .create_async()
            .await;

        (mock_auth_api, mock_auth_server)
    }
}

#[cfg(test)]
mod test_get_calendar_events_for_today {
    use chrono::{DateTime, Datelike, Duration, TimeZone, Utc};
    use google_calendar3::{
        hyper_rustls::HttpsConnector,
        hyper_util::client::legacy::connect::HttpConnector,
        yup_oauth2::{ServiceAccountAuthenticator, ServiceAccountKey, authenticator::Authenticator},
    };
    use mockito::{Mock, ServerGuard};
    use rsa::{RsaPrivateKey, pkcs8::EncodePrivateKey, rand_core::OsRng};

    use super::test_utils::{get_mock_auth_setup, setup_default_crypto_provider};
    use crate::{get_calendar_events_for_today, get_calendar_hub};

    fn get_google_api_formatted_date(date: DateTime<Utc>) -> String {
        date.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
    }

    fn get_start_of_day() -> DateTime<Utc> {
        let now = Utc::now();
        Utc.with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
            .unwrap()
    }

    async fn get_mock_google_api_setup(
        response_body: &String,
        calendar_id: &str,
        start_of_day: DateTime<Utc>,
        end_of_day: DateTime<Utc>,
    ) -> (Mock, ServerGuard) {
        let mut mock_google_api_server = mockito::Server::new_async().await;

        let mock_google_api = mock_google_api_server
            .mock("GET", format!("/calendars/{}/events", calendar_id).as_str())
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("timeMin".into(), get_google_api_formatted_date(start_of_day).into()),
                mockito::Matcher::UrlEncoded("timeMax".into(), get_google_api_formatted_date(end_of_day).into()),
                mockito::Matcher::UrlEncoded("alt".into(), "json".into()),
            ]))
            .with_body(response_body)
            .with_status(200)
            .create_async()
            .await;
        (mock_google_api, mock_google_api_server)
    }

    async fn get_mock_service_account_authenticator(
        mock_auth_server_url: &str,
    ) -> Authenticator<HttpsConnector<HttpConnector>> {
        let private_key_pem = RsaPrivateKey::new(&mut OsRng, 2048)
            .unwrap()
            .to_pkcs8_pem(Default::default())
            .unwrap();
        let service_account_key = ServiceAccountKey {
            key_type: Some("service_account".to_string()),
            project_id: Some("blah".to_string()),
            private_key_id: Some("fake_private_key_id".to_string()),
            private_key: private_key_pem.to_string(),
            client_email: "blah@bob.com".to_string(),
            client_id: Some("fake_client_id".to_string()),
            auth_uri: Some(mock_auth_server_url.to_string()),
            token_uri: mock_auth_server_url.to_string(),
            auth_provider_x509_cert_url: Some(mock_auth_server_url.to_string()),
            client_x509_cert_url: Some(mock_auth_server_url.to_string()),
        };

        ServiceAccountAuthenticator::builder(service_account_key)
            .build()
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn it_returns_empty_list_of_events_if_none_are_present() {
        let _ = env_logger::try_init();
        let start_of_day = get_start_of_day();
        let end_of_day = start_of_day + Duration::days(1);
        let calendar_id = "test_calendar_id";
        let mock_google_api_response = serde_json::json!({
            "items": []
        });

        let (mock_google_api, mock_google_api_server) = get_mock_google_api_setup(
            &mock_google_api_response.to_string(),
            calendar_id,
            start_of_day,
            end_of_day,
        )
        .await;

        let (_mock_auth_api, mock_auth_server) = get_mock_auth_setup().await;
        let mock_service_account_authenticator =
            get_mock_service_account_authenticator(mock_auth_server.url().as_str()).await;

        // Required per test to avoid this error:
        // no process-level CryptoProvider available -- call CryptoProvider::install_default() before this point
        setup_default_crypto_provider();
        let mut hub = get_calendar_hub(mock_service_account_authenticator).unwrap();
        hub.base_url(mock_google_api_server.url() + "/");

        let events = get_calendar_events_for_today(hub, calendar_id).await.unwrap();
        mock_google_api.assert_async().await;
        assert_eq!(events.len(), 0);
    }

    #[tokio::test]
    async fn it_returns_calendar_events_for_today() {
        let _ = env_logger::try_init();
        let start_of_day = get_start_of_day();
        let end_of_day = start_of_day + Duration::days(1);

        let calendar_id = "test_calendar_id";
        let mock_google_api_response = serde_json::json!({
            "items": [
                {
                    "summary": "Holiday",
                    "start": {
                        "dateTime": get_google_api_formatted_date(start_of_day),
                    },
                    "end": {
                        "dateTime": get_google_api_formatted_date(end_of_day),
                    }
                },
                                {
                    "summary": "Lunch",
                    "start": {
                        "dateTime": get_google_api_formatted_date(start_of_day + Duration::hours(12)),
                    },
                    "end": {
                        "dateTime": get_google_api_formatted_date(start_of_day + Duration::hours(13)),
                    }
                }

            ]
        });
        let (mock_google_api, mock_google_api_server) = get_mock_google_api_setup(
            &mock_google_api_response.to_string(),
            calendar_id,
            start_of_day,
            end_of_day,
        )
        .await;

        let (_mock_auth_api, mock_auth_server) = get_mock_auth_setup().await;
        let mock_service_account_authenticator =
            get_mock_service_account_authenticator(mock_auth_server.url().as_str()).await;

        // Required per test to avoid this error:
        // no process-level CryptoProvider available -- call CryptoProvider::install_default() before this point
        setup_default_crypto_provider();
        let mut hub = get_calendar_hub(mock_service_account_authenticator).unwrap();
        hub.base_url(mock_google_api_server.url() + "/");

        let events = get_calendar_events_for_today(hub, calendar_id).await.unwrap();
        mock_google_api.assert_async().await;
        assert_eq!(events.len(), 2);
        for (i, event) in events.iter().enumerate() {
            assert_eq!(
                event.summary.as_deref(),
                Some(mock_google_api_response["items"][i]["summary"].as_str().unwrap())
            );
        }
    }
}

#[cfg(test)]
mod test_get_service_account_authenticator {
    use google_calendar3::yup_oauth2::ServiceAccountKey;
    use rsa::{RsaPrivateKey, pkcs8::EncodePrivateKey, rand_core::OsRng};

    use super::test_utils::{get_mock_auth_setup, setup_default_crypto_provider};
    use crate::{get_calendar_hub, get_service_account_authenticator};

    #[tokio::test]
    async fn it_builds_service_account_authenticator() {
        let _ = env_logger::try_init();

        // Start a mock auth server so the key's URIs are valid HTTP endpoints
        let (_mock_auth, mock_auth_server) = get_mock_auth_setup().await;

        // Generate a temporary RSA private key and create a minimal ServiceAccountKey
        let private_key_pem = RsaPrivateKey::new(&mut OsRng, 2048)
            .unwrap()
            .to_pkcs8_pem(Default::default())
            .unwrap();

        let service_account_key = ServiceAccountKey {
            key_type: Some("service_account".to_string()),
            project_id: Some("test_proj".to_string()),
            private_key_id: Some("fake_private_key_id".to_string()),
            private_key: private_key_pem.to_string(),
            client_email: "svc@test.local".to_string(),
            client_id: Some("fake_client_id".to_string()),
            auth_uri: Some(mock_auth_server.url().to_string()),
            token_uri: mock_auth_server.url().to_string(),
            auth_provider_x509_cert_url: Some(mock_auth_server.url().to_string()),
            client_x509_cert_url: Some(mock_auth_server.url().to_string()),
        };

        // Build the authenticator using the library function under test
        let authenticator = get_service_account_authenticator(service_account_key)
            .await
            .expect("failed to build service account authenticator");

        // Install crypto provider required by hyper-rustls client before building a hub
        setup_default_crypto_provider();

        // Ensure the returned authenticator can be consumed by our hub factory
        let _hub = get_calendar_hub(authenticator).expect("failed to build calendar hub");
    }
}

#[cfg(test)]
mod test_error_conversion {
    use google_calendar3::Error as GoogleAPIError;

    use crate::GoogleCalendarIntegrationError;

    #[test]
    fn it_converts_google_api_error_into_integration_error() {
        // Create a simple, easily-constructible Google API error variant
        let api_err = GoogleAPIError::FieldClash("test_field");

        // Convert using the From<GoogleAPIError> impl under test
        let google_calendar_integration_error: GoogleCalendarIntegrationError = api_err.into();

        // Assert the converted error has the expected variant and preserves inner error detail
        match google_calendar_integration_error {
            GoogleCalendarIntegrationError::GoogleCalendarCommunicationError(inner) => match inner.as_ref() {
                GoogleAPIError::FieldClash(s) => assert_eq!(*s, "test_field"),
                other => panic!("unexpected inner GoogleAPIError variant: {:?}", other),
            },
            other => panic!("unexpected outer error variant: {:?}", other),
        }
    }
}
