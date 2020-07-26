# Todos API using Actix-Web

This is a small example app written in [actix](https://actix.rs) to support a simple Todo List application.

## Setup

Copy `.env.example` to `.env` and set up your [database URL](https://github.com/launchbadge/sqlx#connecting). You won't be able to compile this code without it!

```bash
cargo run
```

## Supported features

- Simple authorization scheme using API keys - every `Client` can have multiple API keys with expiration date
- Two-tiered hierarchy - clients have work lists, which in turn contains todos.
- Logging
- Validation of inputs

Your API should now listen on `127.0.0.1:8080`.

## Supported databases

This project supports two databases:

- SQLite for development purposes
- PostgreSQL for production

Please enable feature `"postgres"` if you want to use PostgreSQL database.
