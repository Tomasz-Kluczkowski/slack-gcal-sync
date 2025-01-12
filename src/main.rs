mod logging;

use crate::logging::setup_default_logger;
use anyhow::{Context, Result};
use clap::Parser;
use configuration::{get_application_configuration, read_json_configuration, ApplicationCommandLineArguments};
use gcal_integration::{get_calendar_events_for_today, get_calendar_hub};
use google_calendar3::yup_oauth2::{ServiceAccountAuthenticator, ServiceAccountKey};
use log::info;
use log4rs::config::load_config_file;
use std::path::Path;

async fn run() -> Result<()> {
    let logging_handle = setup_default_logger();

    let args = ApplicationCommandLineArguments::parse();
    let application_config_path = args.application_config_path.clone();

    let application_configuration = get_application_configuration(args).with_context(|| {
        format!(
            "Failed to read application config from path: '{}'",
            application_config_path
        )
    })?;

    if Path::new(application_configuration.logging_config_path.as_str()).exists() {
        info!(
            "Loading logging configuration from path: '{}'",
            application_configuration.logging_config_path
        );
        let config_from_file = load_config_file(
            application_configuration.logging_config_path.as_str(),
            Default::default(),
        )
        .with_context(|| {
            format!(
                "Failed to load logging config at path: '{}'.",
                application_configuration.logging_config_path
            )
        })?;
        logging_handle.set_config(config_from_file);
        info!("Successfully loaded logging configuration.");
    } else {
        info!(
            "No logging configuration found at path: '{}'. Using default.",
            application_configuration.logging_config_path
        );
    }

    let google_api_error_context = || {
        format!(
            "Failed to integrate with Google API using supplied service account key on path '{}'. \
        Check contents of the service account key json file and further details of the error.",
            application_configuration.service_account_key_path
        )
    };

    info!(
        "Reading google calendar service account key from path: '{}'.",
        application_configuration.service_account_key_path
    );
    let service_account_key =
        read_json_configuration::<ServiceAccountKey>(&application_configuration.service_account_key_path)
            .with_context(|| {
                format!(
                    "Failed to read service account key data from path: '{}'",
                    application_configuration.service_account_key_path
                )
            })?;
    info!("Successfully read google calendar service account key.");

    info!("Setting up service account authenticator.");
    let service_account_authenticator = ServiceAccountAuthenticator::builder(service_account_key)
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

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("Application Error:\n{:?}", err);
        std::process::exit(1);
    }
}
