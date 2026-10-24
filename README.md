# Axum Experiment

Rust 2024 REST API using Axum, Tokio, Serde, SQLx/PostgreSQL, and tracing.

## Requirements

- Stable Rust with Cargo, rustfmt, and Clippy
- Docker with Docker Compose

## Local development

Start PostgreSQL and run the API:

```bash
docker compose up -d db
export DATABASE_URL=postgres://app:localdev@localhost:5432/axum_experiment
cargo run
```

The application applies checked-in migrations automatically outside production.
Use `HTTP_HOST`, `HTTP_PORT`, `DATABASE_URL`, `DATABASE_MAX_CONNECTIONS`, and
`RUST_LOG` to configure the service. The development database URL defaults to
the local Compose service; production requires an explicit `DATABASE_URL`.
In production, migrations are not run unless `RUN_MIGRATIONS=true` is set.

Interactive API documentation is available at `http://localhost:3000/swagger-ui`;
the generated OpenAPI document is served at `/api-docs/openapi.json`.

## Checks

```bash
cargo build
cargo test
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

PostgreSQL integration test (uses the configured database and only removes its
own generated row):

```bash
DATABASE_URL=postgres://app:localdev@localhost:5432/axum_experiment \
  cargo test --test postgres -- --ignored
```

## SQLx offline query metadata

SQL statements use SQLx compile-time macros. To update `.sqlx` metadata after
changing a query or migration, start PostgreSQL and install the SQLx CLI, then
run:

```bash
cargo install sqlx-cli --version 0.8.6 --no-default-features --features rustls,postgres
SQLX_OFFLINE=false DATABASE_URL=postgres://app:localdev@localhost:5432/axum_experiment \
  cargo sqlx prepare
SQLX_OFFLINE=true cargo check
```

Commit the generated `.sqlx` directory together with query changes. Never
commit production credentials; the Compose defaults are only for local use.
