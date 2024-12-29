# slack-gcal-sync
Synchronise events from Google Calendar to user profile in Slack

![Lint-Test-Build](https://github.com/Tomasz-Kluczkowski/slack-gcal-sync/actions/workflows/ci.yml/badge.svg)

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
