use anyhow::{Context, Result};
use clap::Parser;
use configuration::{get_application_configuration, read_json_configuration, ApplicationCommandLineArguments};
use gcal_integration::{get_calendar_events_for_today, get_calendar_hub};
use google_calendar3::yup_oauth2::ServiceAccountKey;
use log::info;

const LOGGING_CONFIG_PATH: &str = "config/logging_config.yaml";

async fn run() -> Result<()> {
    log4rs::init_file(LOGGING_CONFIG_PATH, Default::default()).with_context(|| {
        format!(
            "Failed to initialize logging config at path: '{}'.",
            LOGGING_CONFIG_PATH
        )
    })?;

    let args = ApplicationCommandLineArguments::parse();
    let application_config_path = args.application_config_path.clone();

    let application_configuration = get_application_configuration(args).with_context(|| {
        format!(
            "Failed to read application config from path: '{}'",
            application_config_path
        )
    })?;

    info!("Reading google calendar service account key.");
    let service_account_key =
        read_json_configuration::<ServiceAccountKey>(&application_configuration.service_account_key_path)
            .with_context(|| {
                format!(
                    "Failed to read service account key data from path: '{}'",
                    application_configuration.service_account_key_path
                )
            })?;
    info!("Successfully read google calendar service account key.");

    let google_api_error_context = || {
        format!(
            "Failed to integrate with Google API using supplied service account key on path '{}'. \
        Check contents of the service account key json file and further details of the error.",
            application_configuration.service_account_key_path
        )
    };

    info!("Connecting to google api hub.");
    let hub = get_calendar_hub(service_account_key)
        .await
        .with_context(google_api_error_context)?;
    info!("Connected to google api hub.");

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
