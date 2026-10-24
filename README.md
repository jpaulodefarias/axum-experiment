# Axum Experiment

Rust 2024 REST API using Axum, Tokio, Serde, SeaORM/PostgreSQL, and tracing.

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

PostgreSQL integration test (create the test database once; the test applies
SeaORM migrations and removes only its own rows):

```bash
docker compose exec -T db createdb -U app axum_experiment_test
DATABASE_URL=postgres://app:localdev@localhost:5432/axum_experiment_test \
  cargo test --test postgres -- --ignored
```

## Migrations

SeaORM migrations are defined in Rust under `src/infrastructure/migrations.rs`
and run automatically outside production. In production they run only when
`RUN_MIGRATIONS=true`. SeaORM tracks applied migrations in its own migration
table. Start with a new, empty local database: a database previously initialized
by SQLx is not adopted automatically and must be prepared manually before
starting this version of the application. The application never resets or
drops a database automatically.

Never commit production credentials; the Compose defaults are only for local
use.
