#[cfg(test)]
mod test_slack_api_client {
    use mockito::{Mock, ServerGuard};
    use reqwest::Client;
    use serde_json::Value;

    use crate::{
        INVALID_AUTH, ProfileData, ProfileRequestBody, SLACK_USER_PROFILE_GET_ENDPOINT,
        SLACK_USER_PROFILE_SET_ENDPOINT, SlackApiClient, SlackApiError, UNKNOWN_METHOD,
    };

    async fn get_mock_user_profile_setup(
        mock_slack_api_response: &Value,
        mock_auth_token: &str,
        method: &str,
        endpoint: &str,
    ) -> (Mock, ServerGuard) {
        let mut mock_slack_api_server = mockito::Server::new_async().await;
        let mock_slack_api = mock_slack_api_server
            .mock(method, format!("/{}", endpoint).as_str())
            .match_header("authorization", format!("Bearer {}", mock_auth_token).as_str())
            .with_body(mock_slack_api_response.to_string())
            .with_status(200)
            .create_async()
            .await;

        (mock_slack_api, mock_slack_api_server)
    }

    #[tokio::test]
    async fn test_get_user_profile() {
        let _ = env_logger::try_init();
        let mock_slack_api_response = serde_json::json!({
            "ok": true,
            "profile": {
              "status_text": "testing rust messaging",
              "status_emoji": ":mountain_railway:",
              "status_expiration": 123,
            }
        });
        let mock_auth_token = "secret_token";
        let (mock_slack_api, mock_slack_api_server) = get_mock_user_profile_setup(
            &mock_slack_api_response,
            mock_auth_token,
            "GET",
            SLACK_USER_PROFILE_GET_ENDPOINT,
        )
        .await;
        let slack_api_client =
            SlackApiClient::new(mock_slack_api_server.url(), mock_auth_token.to_string(), Client::new());

        let user_profile_response = slack_api_client
            .get_user_profile(SLACK_USER_PROFILE_GET_ENDPOINT)
            .await
            .unwrap();
        mock_slack_api.assert_async().await;

        assert_eq!(user_profile_response.body.ok, mock_slack_api_response["ok"]);
        assert_eq!(user_profile_response.body.error, None);
        assert_eq!(user_profile_response.status, reqwest::StatusCode::OK);
        assert_eq!(
            user_profile_response.url,
            slack_api_client.get_endpoint_url(SLACK_USER_PROFILE_GET_ENDPOINT)
        );
        let inner_profile = user_profile_response.body.profile.unwrap();
        assert_eq!(
            inner_profile.status_text,
            mock_slack_api_response["profile"]["status_text"]
        );
        assert_eq!(
            inner_profile.status_emoji,
            mock_slack_api_response["profile"]["status_emoji"]
        );
        assert_eq!(
            inner_profile.status_expiration,
            mock_slack_api_response["profile"]["status_expiration"]
        );
    }

    #[tokio::test]
    async fn test_set_user_profile() {
        let _ = env_logger::try_init();
        let new_status_text = "new status text";
        let new_status_emoji = ":mountain_railway:";
        let new_status_expiration = 123;
        let mock_slack_api_response = serde_json::json!({
          "ok": true,
          "profile": {
            "status_text": new_status_text,
            "status_emoji": new_status_emoji,
            "status_expiration": new_status_expiration,
          }
        }
        );
        let mock_auth_token = "secret_token";
        let (mock_slack_api, mock_slack_api_server) = get_mock_user_profile_setup(
            &mock_slack_api_response,
            mock_auth_token,
            "POST",
            SLACK_USER_PROFILE_SET_ENDPOINT,
        )
        .await;
        let slack_api_client =
            SlackApiClient::new(mock_slack_api_server.url(), mock_auth_token.to_string(), Client::new());

        let profile_data = ProfileData {
            status_text: new_status_text.to_string(),
            status_emoji: new_status_emoji.to_string(),
            status_expiration: new_status_expiration,
        };
        let profile_request_body = ProfileRequestBody { profile: profile_data };

        let user_profile_response = slack_api_client
            .set_user_profile(SLACK_USER_PROFILE_SET_ENDPOINT, &profile_request_body)
            .await
            .unwrap();
        mock_slack_api.assert_async().await;

        assert_eq!(user_profile_response.body.ok, mock_slack_api_response["ok"]);
        assert_eq!(user_profile_response.body.error, None);
        assert_eq!(user_profile_response.status, reqwest::StatusCode::OK);
        assert_eq!(
            user_profile_response.url,
            slack_api_client.get_endpoint_url(SLACK_USER_PROFILE_SET_ENDPOINT)
        );
        let inner_profile = user_profile_response.body.profile.unwrap();
        assert_eq!(inner_profile.status_text, new_status_text.to_string());
        assert_eq!(inner_profile.status_emoji, new_status_emoji.to_string());
        assert_eq!(inner_profile.status_expiration, new_status_expiration);
    }

    #[tokio::test]
    async fn test_raises_response_error() {
        let _ = env_logger::try_init();

        let mock_auth_token = "secret_token";
        let slack_api_client = SlackApiClient::new("fake_url".to_string(), mock_auth_token.to_string(), Client::new());
        let error = slack_api_client.get_user_profile(SLACK_USER_PROFILE_GET_ENDPOINT).await;
        assert!(matches!(error, Err(SlackApiError::ResponseError { .. })));
        assert!(
            error
                .unwrap_err()
                .to_string()
                .contains("Failed to make request to Slack api: ")
        );
    }

    #[tokio::test]
    async fn test_raises_invalid_auth_error() {
        let _ = env_logger::try_init();
        let mock_slack_api_response = serde_json::json!({
            "ok": false,
            "error": INVALID_AUTH,
        });
        let mock_auth_token = "invalid_token";
        let (mock_slack_api, mock_slack_api_server) = get_mock_user_profile_setup(
            &mock_slack_api_response,
            mock_auth_token,
            "GET",
            SLACK_USER_PROFILE_GET_ENDPOINT,
        )
        .await;

        let slack_api_client =
            SlackApiClient::new(mock_slack_api_server.url(), mock_auth_token.to_string(), Client::new());

        let error = slack_api_client.get_user_profile(SLACK_USER_PROFILE_GET_ENDPOINT).await;
        mock_slack_api.assert_async().await;
        assert!(matches!(error, Err(SlackApiError::InvalidAuthError { .. })));
        assert!(error.unwrap_err().to_string().contains("Invalid authorization token."));
    }

    #[tokio::test]
    async fn test_raises_unknown_slack_api_method_error() {
        // testing reaction to an unknown api usage such as calls to "https://slack.comapiasdl"
        let _ = env_logger::try_init();
        let mock_slack_api_response = serde_json::json!({
            "ok": false,
            "error": UNKNOWN_METHOD,
        });
        let mock_auth_token = "secret_token";
        let unknown_endpoint = "/unknown/endpoint";
        let (mock_slack_api, mock_slack_api_server) =
            get_mock_user_profile_setup(&mock_slack_api_response, mock_auth_token, "GET", unknown_endpoint).await;

        let slack_api_client =
            SlackApiClient::new(mock_slack_api_server.url(), mock_auth_token.to_string(), Client::new());

        let error = slack_api_client.get_user_profile(unknown_endpoint).await;
        mock_slack_api.assert_async().await;
        assert!(matches!(error, Err(SlackApiError::UnknownSlackApiMethodError { .. })));
        assert_eq!(
            error.unwrap_err().to_string(),
            format!(
                "Unknown Slack api called: {}/{}. Check request URL matches list of available apis: https://api.slack.com/apis.",
                mock_slack_api_server.url(),
                unknown_endpoint
            )
        );
    }

    #[tokio::test]
    async fn test_raises_slack_api_error_for_all_other_errors() {
        let _ = env_logger::try_init();
        let error_message = "some other error in slack api occurred";

        let mock_slack_api_response = serde_json::json!({
            "ok": false,
            "error": error_message,
        });
        let mock_auth_token = "secret_token";
        let (mock_slack_api, mock_slack_api_server) = get_mock_user_profile_setup(
            &mock_slack_api_response,
            mock_auth_token,
            "GET",
            SLACK_USER_PROFILE_GET_ENDPOINT,
        )
        .await;

        let slack_api_client =
            SlackApiClient::new(mock_slack_api_server.url(), mock_auth_token.to_string(), Client::new());

        let error = slack_api_client.get_user_profile(SLACK_USER_PROFILE_GET_ENDPOINT).await;
        mock_slack_api.assert_async().await;
        assert!(matches!(error, Err(SlackApiError::SlackAPIError { .. })));
        assert_eq!(
            error.unwrap_err().to_string(),
            format!("Error calling Slack API: {}", error_message)
        );
    }
}
