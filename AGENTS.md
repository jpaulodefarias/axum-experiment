# Agent Instructions

## Project source of truth

- Read `SDD.md` before making architectural or API changes. Keep implementation,
  this file, and `README.md` consistent with it.
- This is a Rust 2024 REST API using Axum, Tokio, Serde, PostgreSQL, SeaORM,
  tracing, Utoipa, Swagger UI, and Cargo. Run Rust tooling locally; Docker
  Compose provides PostgreSQL. There is no Dev Container workflow.
- Keep `Cargo.lock` committed with application and database changes.

## Architecture and Rust style

- Keep domain and application code independent of Axum, SeaORM, PostgreSQL, and
  HTTP/JSON transport types. Define application-facing repository ports in an
  inner layer and implement them in infrastructure.
- Keep Axum handlers focused on HTTP parsing/response mapping. Put business
  behavior in application use cases and wire concrete dependencies explicitly
  in the composition root.
- Use named-field structs for state, dependencies, and values with distinct
  meanings. Use named enum-variant fields for meaningful payloads. Reserve
  tuple structs/variants for intentional newtypes or other cases where
  position is the type's purpose; framework response tuples are acceptable
  where Axum uses them as its response idiom.
- Prefer typed errors and explicit `Result` handling. Do not expose SeaORM or
  PostgreSQL errors to clients, and avoid `unwrap`/`expect` in request and
  infrastructure paths.
- Keep OpenAPI annotations beside the handlers and DTOs they describe. Update
  schemas and documented responses whenever the public HTTP API changes.
- Do not add dependencies or abstractions without a concrete need. Do not
  introduce unsafe Rust without a documented, narrowly scoped reason.

## Database and configuration

- Use SeaORM's parameterized query APIs. Add schema changes as versioned
  SeaORM migrations. Local development may apply migrations automatically;
  production must not migrate implicitly. Never
  commit secrets or use the Compose credentials outside local development.
- The local database is started with `docker compose up -d db`; the app runs
  directly with Cargo.

## HTTP and errors

- Preserve the resource-oriented `/api/v1` conventions in `SDD.md` and the
  standard error shape (`code`, `message`, `traceId`). Keep `traceId` aligned
  with the `x-request-id` response header.
- `DELETE /api/v1/users` deletes the entire collection. Treat it as a
  destructive operation; do not invoke it against a database containing user
  data as part of routine implementation or verification.
- Swagger UI is served at `/swagger-ui`; the generated OpenAPI document is at
  `/api-docs/openapi.json`.

## Development checks

Run relevant checks after changes. The standard local checks are:

```bash
cargo build
cargo test
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

PostgreSQL integration tests are ignored in the default test run and require
an explicitly configured local test database:

```bash
DATABASE_URL=postgres://app:localdev@localhost:5432/axum_experiment \
  cargo test --test postgres -- --ignored
```

Add or update focused tests when changing behavior. Keep database-backed tests
separate from unit and in-process HTTP tests, and ensure they clean up only the
records they create.
