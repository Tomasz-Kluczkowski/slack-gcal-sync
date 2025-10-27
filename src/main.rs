use anyhow::{Context, Result};
use chrono::Utc;
use clap::Parser;
use configuration::{ApplicationConfigurationData, ApplicationConfigurationGetter};
use gcal_integration::{get_calendar_events_for_today, get_calendar_hub};
use google_calendar3::yup_oauth2::ServiceAccountAuthenticator;
use log::info;
use logging::LoggerConfigurator;
use reqwest::Client;
use slack_integration::{
    ProfileData, ProfileRequestBody, SlackApiClient, SLACK_API_BASE_URL, SLACK_USER_PROFILE_GET_ENDPOINT,
    SLACK_USER_PROFILE_SET_ENDPOINT,
};
use std::path::Path;

async fn run() -> Result<()> {
    info!("Starting application.");
    info!("Setting up default logging.");
    let mut logger_configurator = LoggerConfigurator::default();
    let logging_handle = logger_configurator.setup_default_logger();

    info!("Parsing command line arguments.");
    let cli_application_configuration_data = ApplicationConfigurationData::parse();
    info!("Successfully parsed command line arguments.");

    info!("Loading application configuration.");
    let application_configuration_getter = ApplicationConfigurationGetter::new(cli_application_configuration_data);
    let application_configuration = application_configuration_getter?.get_application_configuration()?;
    info!("Successfully loaded application configuration.");

    if Path::new(application_configuration.logging_config_path.as_str()).exists() {
        info!(
            "Loading logging configuration from path: '{}'",
            application_configuration.logging_config_path
        );
        logger_configurator
            .apply_logging_config_from_file(application_configuration.logging_config_path.as_str(), &logging_handle)?;
        info!("Successfully loaded logging configuration.");
    } else {
        info!(
            "No logging configuration found at path: '{}'. Using default.",
            application_configuration.logging_config_path
        );
    }

    let google_api_error_context = || "Failed to integrate with Google API using supplied service account key.";

    info!("Setting up service account authenticator.");
    let service_account_authenticator =
        ServiceAccountAuthenticator::builder(application_configuration.service_account_key)
            .build()
            .await
            .with_context(google_api_error_context)?;
    info!("Successfully set up up service account authenticator.");

    info!("Setting up google api hub.");
    let hub = get_calendar_hub(service_account_authenticator)
        .await
        .with_context(google_api_error_context)?;
    info!("Successfully set up google api hub.");

    info!("Fetching google calendar events for today.");
    let events = get_calendar_events_for_today(hub, application_configuration.calendar_id.as_str())
        .await
        .with_context(google_api_error_context)?;
    info!(
        "Successfully fetched {} google calendar events for today.",
        events.len()
    );

    for event in events {
        println!("event summary: {:#?}", event.summary);
        println!("event start time: {:#?}", event.start);
        println!("event end time: {:#?}", event.end);
        println!("\n");
    }
    // Testing reading Slack user profile
    let slack_api_client = SlackApiClient::new(
        SLACK_API_BASE_URL.to_owned(),
        application_configuration.slack_user_oauth_token,
        Client::new(),
    );
    let slack_user_profile = slack_api_client
        .get_user_profile(SLACK_USER_PROFILE_GET_ENDPOINT)
        .await?;
    println!("Previous Slack user profile: {:#?}", slack_user_profile);
    println!("Status: {}", slack_user_profile.status);
    println!("Url: {}", slack_user_profile.url);
    println!("OK: {}", slack_user_profile.body.ok);
    println!("Profile: {:?}", slack_user_profile.body.profile);

    // Testing setting Slack user profile
    let profile_data = ProfileData {
        status_text: format!("testing rust messaging on: {}", Utc::now()),
        status_emoji: ":mountain_railway:".to_string(),
        status_expiration: 0,
    };

    let profile_request_body = ProfileRequestBody { profile: profile_data };
    let slack_new_user_profile = slack_api_client
        .set_user_profile(SLACK_USER_PROFILE_SET_ENDPOINT, &profile_request_body)
        .await?;
    println!("New Slack user profile: {:#?}", slack_new_user_profile);

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("Application Error:\n{:?}", err);
        std::process::exit(1);
    }
}
