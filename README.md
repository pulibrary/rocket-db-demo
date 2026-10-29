# Rocket DB Demo

## Setup

1. [Install Devbox](https://www.jetify.com/devbox)
2. `devbox shell`
3. `devbox services up` (-b to run in the background)

### Databases

The demo app does not currently create a database for SQLx
You will need to create it on the psql client
1. `psql -U postgres`
2. `create database rocket_db_demo owner postgres;`

## SQLx

Install the SQLx command line tool
`cargo install sqlx-cli --no-default-features --features postgres`

SQLx requires either a running database or to cache metadata for the schema.
The metadata is cached in the .sqlx directory and the current schema is already cached.

To regenerate it after schema changes run

`sqlx prepare --database-url "postgres://postgres:postgres@localhost:5432/rocket_db_demo"`

## Diesel

The Diesel ORM automatic migrations on startup isn't quite working and I've commented out the inclusion in the main.rs file. 