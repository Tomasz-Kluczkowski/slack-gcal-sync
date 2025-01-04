mod tests;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use thiserror::Error;

// TODO: check if these constants actually need to be public, they probably don't now when this is a separate library.
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

const SLACK_USER_PROFILE_SET_URL: &str = "https://slack.com/api/users.profile.set";
const SLACK_USER_PROFILE_GET_URL: &str = "https://slack.com/api/users.profile.get";
// const SLACK_USER_PROFILE_SET_URL: &str = "https://slack.comapiasdl"; // broken url to cause an unknown method error
// const SLACK_USER_PROFILE_GET_URL: &str = "https://slack.com/api/asd"; // broken url to cause an unknown_method error
// const SLACK_USER_PROFILE_GET_URL: &str = "https://slack.co1m/api"; // broken url to cause a reqwest connect error

#[derive(Debug, Serialize, Deserialize)]
struct ProfileData {
    status_text: String,
    status_emoji: String,
    status_expiration: i64,
}

#[derive(Debug, Serialize)]
struct ProfileRequestBody {
    profile: ProfileData,
}

#[derive(Debug, Deserialize)]
struct ProfileResponseBody {
    ok: bool,
    profile: Option<ProfileData>,
    error: Option<String>,
}

#[derive(Debug)]
struct ProfileResponse {
    status: reqwest::StatusCode,
    url: String,
    body: ProfileResponseBody,
}

async fn get_user_profile(auth_token: String, client: &Client) -> Result<ProfileResponse, SlackApiError> {
    let response = client
        .get(SLACK_USER_PROFILE_GET_URL)
        .bearer_auth(auth_token)
        .send()
        .await;

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
                    INVALID_AUTH => return Err(SlackApiError::InvalidAuthError),
                    UNKNOWN_METHOD => return Err(SlackApiError::UnknownSlackApiMethodError(profile_response.url)),
                    _ => return Err(SlackApiError::SlackAPIError(error.to_string())),
                },
                None => Ok(profile_response),
            }
        }
        Err(error) => Err(SlackApiError::ResponseError(error)),
    }
}

async fn set_user_profile(auth_token: String, client: &Client) -> Result<ProfileResponse, SlackApiError> {
    // let profile_data = ProfileRequestData {
    //     status_text: "testing rust messaging".to_string(),
    //     status_emoji: ":mountain_railway:".to_string(),
    //     status_expiration: 0,
    // };

    // let profile_body = ProfileRequestBody {
    //     profile: profile_data
    // };

    // let token = "fake";
    // let response = client
    //     .post(SLACK_USER_PROFILE_SET_URL)
    //     .bearer_auth(token)
    //     // .json(&profile_data)
    //     .json(&profile_body)
    //     .send()
    //     .await;

    // let response = client
    //     .get(SLACK_USER_PROFILE_GET_URL)
    //     .bearer_auth(token)
    //     .send()
    //     .await;
    //
    // match response {
    //     Ok(response) => {
    //         println!("Status: {}", response.status());
    //         let response_body = response.text().await?;
    //         println!("Response: \n{}", response_body);
    //     }
    //     Err(error) => {
    //         println!("Error: {}", error)
    //     }
    // }

    let response = client
        .get(SLACK_USER_PROFILE_GET_URL)
        .bearer_auth(auth_token)
        .send()
        .await;

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
                    INVALID_AUTH => return Err(SlackApiError::InvalidAuthError),
                    UNKNOWN_METHOD => return Err(SlackApiError::UnknownSlackApiMethodError(profile_response.url)),
                    _ => return Err(SlackApiError::SlackAPIError(error.to_string())),
                },
                None => Ok(profile_response),
            }
        }
        Err(error) => Err(SlackApiError::ResponseError(error)),
    }
}

async fn main() -> Result<(), SlackApiError> {
    // TODO: read this from a file same as service account key and override via env var. No command line arg.  Move this to get app configuration
    let token = env::var("USER_AUTH_TOKEN").expect("USER_AUTH_TOKEN env variable not found.");
    // let token: String = "fake".to_string();
    let client = Client::new();

    match get_user_profile(token, &client).await {
        Ok(profile) => {
            println!("Successfully got profile {:#?}.", profile);
        }
        Err(error) => {
            eprintln!("Failed to get profile: {}", error);
        }
    }

    // let profile_data = ProfileRequestData {
    //     status_text: "testing rust messaging".to_string(),
    //     status_emoji: ":mountain_railway:".to_string(),
    //     status_expiration: 0,
    // };

    // let profile_body = ProfileRequestBody {
    //     profile: profile_data
    // };

    // let token = "fake";
    // let response = client
    //     .post(SLACK_USER_PROFILE_SET_URL)
    //     .bearer_auth(token)
    //     // .json(&profile_data)
    //     .json(&profile_body)
    //     .send()
    //     .await;

    // let response = client
    //     .get(SLACK_USER_PROFILE_GET_URL)
    //     .bearer_auth(token)
    //     .send()
    //     .await;
    //
    // match response {
    //     Ok(response) => {
    //         println!("Status: {}", response.status());
    //         let response_body = response.text().await?;
    //         println!("Response: \n{}", response_body);
    //     }
    //     Err(error) => {
    //         println!("Error: {}", error)
    //     }
    // }

    //

    Ok(())
}
