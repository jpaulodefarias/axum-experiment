# ADR-009: Use SeaORM for PostgreSQL persistence

- Status: accepted
- Date: 2026-10-24
- Supersedes: ADR-004's choice of SQLx as the application's database API

## Context

The application uses SQLx directly for its PostgreSQL pool, repository queries,
and migrations. The repository port already isolates persistence from the
application and HTTP layers. The service is a single small crate and currently
runs pending migrations from its startup composition root.

## Decision

Use SeaORM for the PostgreSQL connection, entity mapping, repository queries,
and versioned migrations. Keep the migration definitions in the application
crate and run them through the existing environment policy: automatically
outside production, and in production only when `RUN_MIGRATIONS=true`.

Keep the `UserRepository` port, domain model, and public HTTP contract
independent of SeaORM. Do not add a separate migration crate or CLI at this
stage. Remove direct SQLx usage and SQLx offline metadata; SeaORM continues to
use SQLx internally for its PostgreSQL driver.

## Consequences

- SeaORM owns the database connection pool and a separate migration history
  table.
- A database initialized by SQLx is not adopted automatically. Before starting
  this version, use a new empty local database or prepare an existing local
  database manually. Startup never resets or drops a database.
- Rust compilation checks entity and migration code, replacing SQLx query
  metadata generation.
- The application remains one Cargo package; a dedicated migration runner can
  be added later if deployment needs require it.

## Alternatives considered

- Keep SQLx for repository access and adopt only SeaORM migrations: rejected
  because the goal is a full replacement of direct SQLx use.
- Put migrations in a separate workspace crate: rejected because the current
  service runs them only from application startup and has no independent
  migration deployment command.
