use anyhow::{Context, Result};
use clap::Parser;
use configuration::{read_json_configuration, ApplicationCommandLineArguments, ApplicationConfiguration};
use gcal_integration::{get_calendar_events_for_today, get_calendar_hub};
use google_calendar3::yup_oauth2::ServiceAccountKey;

async fn run() -> Result<()> {
    let args = ApplicationCommandLineArguments::parse();

    let application_configuration = ApplicationConfiguration {
        service_account_key_path: args.service_account_key_path,
        calendar_id: args.calendar_id,
    };

    let service_account_key =
        read_json_configuration::<ServiceAccountKey>(&application_configuration.service_account_key_path)
            .with_context(|| {
                format!(
                    "Failed to read service account key data from path: '{}'",
                    application_configuration.service_account_key_path
                )
            })?;

    let google_api_error_context = || {
        format!(
            "Failed to integrate with Google API using supplied service account key on path '{}'. \
        Check contents of the service account key json file and further details of the error.",
            application_configuration.service_account_key_path
        )
    };
    let hub = get_calendar_hub(service_account_key)
        .await
        .with_context(google_api_error_context)?;

    let events = get_calendar_events_for_today(hub, application_configuration.calendar_id.as_str())
        .await
        .with_context(google_api_error_context)?;

    for event in events.unwrap() {
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
