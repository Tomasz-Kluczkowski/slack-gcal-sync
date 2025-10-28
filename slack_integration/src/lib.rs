mod tests;

use reqwest::{Client, Response, Result as ReqwestResult};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const INVALID_AUTH: &str = "invalid_auth";
pub const UNKNOWN_METHOD: &str = "unknown_method";

#[derive(Debug, Error)]
pub enum SlackApiError {
    #[error("Invalid authorization token.")]
    InvalidAuthError,

    #[error(
        "Unknown Slack api called: {0}. Check request URL matches list of available apis: https://api.slack.com/apis."
    )]
    UnknownSlackApiMethodError(String),

    #[error("Error calling Slack API: {0}")]
    SlackAPIError(String),

    #[error("Failed to make request to Slack api: {0}")]
    ResponseError(#[from] reqwest::Error),
}

pub const SLACK_API_BASE_URL: &str = "https://slack.com/api";
pub const SLACK_USER_PROFILE_GET_ENDPOINT: &str = "users.profile.get";
pub const SLACK_USER_PROFILE_SET_ENDPOINT: &str = "users.profile.set";

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ProfileData {
    pub status_text: String,
    pub status_emoji: String,
    pub status_expiration: i64,
}

#[derive(Debug, Serialize)]
pub struct ProfileRequestBody {
    pub profile: ProfileData,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct ProfileResponseBody {
    pub ok: bool,
    pub profile: Option<ProfileData>,
    pub error: Option<String>,
}

#[derive(Debug)]
pub struct ProfileResponse {
    pub status: reqwest::StatusCode,
    pub url: String,
    pub body: ProfileResponseBody,
}

pub struct SlackApiClient {
    api_base_url: String,
    auth_token: String,
    client: Client,
}

impl SlackApiClient {
    pub fn new(api_base_url: String, auth_token: String, client: Client) -> SlackApiClient {
        SlackApiClient {
            api_base_url,
            auth_token,
            client,
        }
    }

    pub fn get_endpoint_url(&self, endpoint: &str) -> String {
        format!("{}/{}", self.api_base_url, endpoint)
    }

    async fn handle_response(&self, response: ReqwestResult<Response>) -> Result<ProfileResponse, SlackApiError> {
        match response {
            Ok(response) => {
                let status = response.status();
                let url = response.url().to_string();
                let response_body: ProfileResponseBody = response.json().await?;
                let profile_response = ProfileResponse {
                    status,
                    url,
                    body: response_body,
                };

                match profile_response.body.error {
                    Some(error) => match error.as_str() {
                        INVALID_AUTH => Err(SlackApiError::InvalidAuthError),
                        UNKNOWN_METHOD => Err(SlackApiError::UnknownSlackApiMethodError(profile_response.url)),
                        _ => Err(SlackApiError::SlackAPIError(error.to_string())),
                    },
                    None => Ok(profile_response),
                }
            }
            Err(error) => Err(SlackApiError::ResponseError(error)),
        }
    }

    pub async fn get_user_profile(&self, endpoint: &str) -> Result<ProfileResponse, SlackApiError> {
        let response = self
            .client
            .get(self.get_endpoint_url(endpoint))
            .bearer_auth(self.auth_token.as_str())
            .send()
            .await;

        self.handle_response(response).await
    }

    pub async fn set_user_profile(
        &self,
        endpoint: &str,
        profile_request_body: &ProfileRequestBody,
    ) -> Result<ProfileResponse, SlackApiError> {
        let response = self
            .client
            .post(self.get_endpoint_url(endpoint))
            .bearer_auth(self.auth_token.as_str())
            .json(&profile_request_body)
            .send()
            .await;

        self.handle_response(response).await
    }
}
