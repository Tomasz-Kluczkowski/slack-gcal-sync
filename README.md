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
