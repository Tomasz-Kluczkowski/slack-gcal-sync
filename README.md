# slack-gcal-sync
Synchronise events from Google Calendar to user profile in Slack

![Lint-Test-Build](https://github.com/Tomasz-Kluczkowski/slack-gcal-sync/actions/workflows/ci.yml/badge.svg)

[![codecov](https://codecov.io/github/Tomasz-Kluczkowski/slack-gcal-sync/graph/badge.svg?token=8AOYDGEMK7)](https://codecov.io/github/Tomasz-Kluczkowski/slack-gcal-sync)

This project is my test ground for learning Rust language.
The goal is as follows:
- get google calendar events on a predetermined interval
- using configuration mapping google calendar event title to slack user profile settings set the new value for slack user profile

## Project Setup

Assuming linux Ubuntu as OS or WSL2 with Ubuntu.

- clone the repo to your local machine
- [install rust language](https://www.rust-lang.org/tools/install)
- open terminal in root of the project
- install pre-commit tool:
```shell
sudo apt update && sudo apt install -y pre-commit
```
- install pre-commit hook for the project:
```shell
pre-commit install
```
- run tests for entire project:
```shell
 cargo test --workspace
```
- build entire project (in debug, unoptimised mode), check `target/debug`:
```shell
cargo build --workspace
```
- build entire project (optimised for production), check `target/release`:
```shell
cargo build --release --workspace
```

## Command line interface

- Run `slack-gcal-sync --help` to see command line interface options.

## Application Configuration

The application configuration is read in the following order:
- if `--application-config-path` cli option is specified, we try to load it from a `json` file at that path.
  The application configuration must specify all values for application configuration, or it will fail to load.
- if `--application-config-path` cli option is **not** specified, a default application path will be used: `config/application_config.json`.
- if there is no application config file (at default or specified path) we will set default app config.
- any CLI options such as `--calendar-id` override what is specified in the application config file or default application config.

### Application config file

This file holds application configuration in json format. All keys are required for configuration to be valid.

```json
{
  "calendar_id":  "my-calendar@gmail.com",
  "service_account_key_path": ".secrets/.service_account.json"
}
```

## Setting Up Integration With Slack API

- navigate to Slack API new app page: https://api.slack.com/apps?new_app=1
- Click `Create New App`

![image](docs/slack_api_integration/images/create_new_slack_app.png)

- Click `From a Manifest`

![image](docs/slack_api_integration/images/create_app_from_a_manifest.png)

- select target workspace for the new app and click `Next`

![image](docs/slack_api_integration/images/pick_workspace_for_new_app.png)

- Paste this manifest in the `Json` tab to allow user profile reading and writing (only) and click `Next`

```json
{
    "display_information": {
        "name": "user-profile-app",
        "description": "User Profile Integration",
        "background_color": "#004492"
    },
    "oauth_config": {
        "scopes": {
            "user": [
                "users.profile:read",
                "users.profile:write"
            ]
        }
    },
    "settings": {
        "org_deploy_enabled": false,
        "socket_mode_enabled": false,
        "token_rotation_enabled": false
    }
}
```

![image](docs/slack_api_integration/images/paste_json_app_manifest.png)


- click `Create`

![image](docs/slack_api_integration/images/review_summary_and_create_app.png)

- After your app is created, navigate to `Settings -> Install App` to install it in your workspace and generate necessary `OAuth tokens` for programmatic communication.

![image](docs/slack_api_integration/images/install_app_in_workspace.png)

- Click `Install to <your workspace name>` (here my workspace is called `lab` so it shows as `Install to lab`).
- Confirm that you want to install the app and click `Allow`.

![image](docs/slack_api_integration/images/confirm_app_installation_in_workspace.png)

- After you click `Allow` you will be presented with the `User OAuth Token` which will be used for programmatic communication with Slack API on behalf of your user. The secret always starts with `xoxp`.

![image](docs/slack_api_integration/images/view_oauth_token.png)

- This can also be found in `Features -> OAuth & Permissions`. 