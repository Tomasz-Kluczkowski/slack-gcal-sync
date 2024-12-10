# slack-gcal-sync
![Lint-Test-Build](https://github.com/Tomasz-Kluczkowski/slack-gcal-sync/actions/workflows/ci.yml/badge.svg)
Synchronise events from Google Calendar to user profile in Slack

This project is my test ground for learning Rust language.
The goal is as follows:
- get google calendar events on a predetermined interval
- using configuration mapping google calendar event title to slack user profile settings set the new value for slack user profile

## Project Setup

Assuming linux Ubuntu as OS or WSL2 with Ubuntu.

- cloning the repo to your local machine
- open terminal in root of the project
- install pre-commit tool:
```shell
sudo apt update && sudo apt install -y pre-commit
```
- install pre-commit hook for the project:
```shell
pre-commit install
```
