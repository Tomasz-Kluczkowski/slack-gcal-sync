#[cfg(test)]
mod get_calendar_events_for_today {
    use crate::{get_calendar_events_for_today, get_calendar_hub};
    use chrono::{DateTime, Datelike, Duration, TimeZone, Utc};
    use google_calendar3::yup_oauth2::read_service_account_key;

    fn get_google_api_formatted_date(date: DateTime<Utc>) -> String {
        date.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
    }

    #[tokio::test]
    async fn it_returns_empty_list_of_events_if_none_are_present() {}

    #[tokio::test]
    async fn test_get_calendar_events_for_today() {
        let _ = env_logger::try_init();
        let now = Utc::now();
        let start_of_day = Utc
            .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
            .unwrap();
        let end_of_day = start_of_day + Duration::days(1);

        let calendar_id = "test_calendar_id";
        let mock_response = serde_json::json!({
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
        let mut server = mockito::Server::new_async().await;

        let mock = server
            .mock("GET", format!("/calendars/{}/events", calendar_id).as_str())
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("timeMin".into(), get_google_api_formatted_date(start_of_day).into()),
                mockito::Matcher::UrlEncoded("timeMax".into(), get_google_api_formatted_date(end_of_day).into()),
                mockito::Matcher::UrlEncoded("alt".into(), "json".into()),
            ]))
            .with_body(mock_response.to_string())
            .with_status(200)
            .create_async()
            .await;

        // TODO: THIS NEEDS ELIMINATING - CHANGE get_calendar_hub to take auth as a parameter instead and inject fake auth.
        let sak = read_service_account_key("../.secrets/.service_account.json")
            .await
            .unwrap();

        rustls::crypto::ring::default_provider().install_default().unwrap();
        let mut hub = get_calendar_hub(sak).await.unwrap();
        hub.base_url(server.url() + "/");

        let events = get_calendar_events_for_today(hub, calendar_id).await.unwrap();
        mock.assert_async().await;
        assert_eq!(events.len(), 2);
        for (i, event) in events.iter().enumerate() {
            assert_eq!(
                event.summary.as_deref(),
                Some(mock_response["items"][i]["summary"].as_str().unwrap())
            );
        }
    }
}
