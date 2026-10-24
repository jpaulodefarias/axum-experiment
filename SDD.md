# Software Design Document — Rust REST API

**Status:** Draft  
**Version:** 0.2  
**Language:** Rust 2024 edition (minimum stable toolchain supporting the edition)  
**Architecture:** Pragmatic Clean Architecture / Hexagonal-inspired  
**HTTP framework:** Axum

---

# 1. Overview

This document describes the initial technical design for the same REST API
project, now implemented in Rust. Its initial scope is project bootstrap, a
minimal HTTP API, PostgreSQL integration, observability, and a first reference
vertical slice.

The implementation should prioritize maintainability, testability, explicit
dependencies, clear separation between business logic and infrastructure,
predictable builds, and a productive Rust development workflow. Application
logic should remain independent of Axum, SeaORM, PostgreSQL, HTTP, and JSON.

# 2. Goals

The initial version must provide:

- a REST API using Axum and Tokio;
- Rust with Cargo as the build, dependency, and workspace tool;
- JSON serialization and deserialization using Serde;
- PostgreSQL access through SeaORM;
- structured application and request logging with tracing;
- a health-check endpoint;
- a pragmatic layered architecture with explicit dependency wiring;
- Docker Compose for local PostgreSQL infrastructure;
- automated unit and integration test support using Rust's built-in test framework;
- database migrations managed by SeaORM Migration;
- formatting and linting through rustfmt and Clippy.

The architecture should allow business features to be added without coupling
domain or application logic to Axum, SeaORM, HTTP, JSON serialization, or
operating-system APIs.

# 3. Non-goals

The first iteration will not implement authentication, authorization,
distributed tracing, Kafka, Redis, Kubernetes, service mesh, advanced caching,
CQRS, event sourcing, complex domain logic, a generic dependency-injection
container, or premature framework abstractions. OpenTelemetry and metrics may
be introduced later by explicit design decision; `tracing` here means
structured in-process instrumentation and logging.

# 4. Technology Stack

| Concern | Choice |
| --- | --- |
| Language | Rust, 2024 edition |
| Async runtime | Tokio |
| HTTP | Axum |
| Serialization | Serde + serde_json |
| Database | PostgreSQL |
| Database client and migrations | SeaORM |
| Logging/instrumentation | tracing + tracing-subscriber |
| API specification and UI | OpenAPI via Utoipa + Swagger UI |
| Build and dependencies | Cargo |

Use stable Rust. Keep dependency versions current and mutually compatible when
the project is bootstrapped, and commit `Cargo.lock` for the application.
Prefer well-supported crates and add dependencies only where they meet a
clear requirement.

Initial dependencies are expected to include `axum`, `tokio`, `serde`,
`serde_json`, `sea-orm` with PostgreSQL and Tokio runtime support,
`sea-orm-migration`, `tracing`, and `tracing-subscriber`. Add crates such as
`thiserror`, `uuid`, or `chrono` only as needed by the implementation. Avoid
adding a separate logging facade when `tracing` meets the requirement.

# 5. Runtime and HTTP Framework

Tokio provides asynchronous task execution and runtime services. Axum handles
HTTP routing, extractors, middleware/layers, and response construction. The
application should have one clearly configured Tokio runtime, normally created
by `#[tokio::main]` in the composition root.

Axum and Tokio types belong in the presentation/runtime boundary. Handlers
should be thin: extract and validate transport data, call an application use
case, then map its result to an HTTP response. Avoid passing `axum::extract::State`,
HTTP request/response types, or Tokio-specific handles into domain code.

# 6. Cargo and Project Organization

Cargo is responsible for builds, dependency resolution, tests, formatting, and
linting. Use the standard `Cargo.toml`, `Cargo.lock`, and `src/` layout. A
single application crate is sufficient initially; split into workspace crates
only when real ownership or reuse needs justify it.

Typical commands:

```bash
cargo build
cargo test
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

Keep configuration and dependency features explicit in `Cargo.toml`. SeaORM
entities and migrations are checked by Rust compilation; do not commit
credentials in configuration.

# 7. Database

PostgreSQL is the primary relational database. SeaORM provides the connection
pool, query API, transactions, and versioned migrations. Create a
`DatabaseConnection` during startup and pass it to infrastructure components
that need it. Pool sizing, timeouts, and connection settings belong in
configuration, not business logic.

Expected dependency direction:

```text
Application use case
        |
        v
Repository port
        ^
        |
Postgres repository (SeaORM)
        |
        v
PostgreSQL
```

Repository interfaces/ports belong to an inner layer. SeaORM entities,
queries, and row mapping belong to infrastructure. Do not put persistence code
in domain entities or use cases. Use SeaORM's parameterized query APIs; never
interpolate untrusted input into SQL strings.

Run migrations as an explicit startup/deployment step according to environment
policy. Automatic schema creation is not permitted in production. The local
development workflow may apply checked-in migrations automatically.

# 8. Architecture

Use a pragmatic Clean Architecture approach. Dependencies point inward:

```text
Presentation (Axum handlers / HTTP DTOs)
        |
        v
Application (use cases / ports)
        |
        v
Domain (models / business rules)

Infrastructure (SeaORM / PostgreSQL / logging adapters)
        └── implements ports defined by inner layers
```

Request flow:

```text
HTTP request -> Axum handler -> application use case -> domain behavior
                                      |
                                      v
                               repository port
                                      ^
                                      |
                           SeaORM repository -> PostgreSQL
```

Rust module visibility and explicit construction should support the intended
boundaries without introducing a generic dependency-injection framework. The
composition root wires concrete dependencies and creates the Axum router.

# 9. Layers

## 9.1 Domain

Contains business concepts and rules. Expected modules may include:

```text
domain/
├── model/
├── repository/
├── service/
└── error/
```

Domain code must not depend on Axum, Tokio, SeaORM, PostgreSQL, Serde, or HTTP.
Use ordinary Rust types and domain-specific errors. Add behavior to entities
when meaningful rules exist; avoid abstractions without a current need.

## 9.2 Application

Contains use cases, commands/results, and ports:

```text
application/
├── dto/
├── usecase/
└── port/
```

Use cases orchestrate application behavior, validate application-level rules,
invoke domain behavior, and call repository ports. They must not know about
HTTP, Axum handlers, SQL, SeaORM pool types, or PostgreSQL connection objects.

Use async functions where I/O is involved. Keep async traits and trait-object
choices proportionate to the code: use a small port trait and a suitable async
trait approach when mocking or substitutability is needed; do not add async
trait machinery solely in anticipation of hypothetical adapters.

## 9.3 Repository Ports

Ports normally live in the application or domain layer, according to whether
they express a business concept or a use-case need. Example (illustrative):

```rust
pub trait UserRepository: Send + Sync {
    async fn save(&self, user: &User) -> Result<User, RepositoryError>;
    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, RepositoryError>;
}
```

The exact trait syntax may use native async functions in traits or a compatible
crate depending on object-safety and mocking requirements. Keep SeaORM types
out of the port contract.

## 9.4 Infrastructure

Infrastructure implements external-system adapters:

```text
infrastructure/
├── database/
│   └── postgres/
├── repository/
└── logging/
```

It owns SeaORM queries, row mapping, migrations, PostgreSQL error translation,
and concrete repository implementations. Translate database errors into
application/domain-safe errors; do not leak SeaORM or raw PostgreSQL errors to
handlers or clients.

## 9.5 Presentation

Axum HTTP code belongs to `presentation/http/`. Responsibilities include
routing, request extraction and validation, mapping HTTP DTOs to application
commands, invoking use cases, producing responses, and translating errors to
HTTP status codes. Handlers should contain minimal business logic.

# 10. State and Dependency Wiring

Use explicit construction in the composition root (`main.rs` or a dedicated
bootstrap module). Axum shared state should hold only the application-facing
services/use cases needed by handlers. Prefer concrete types where practical;
use `Arc` when shared ownership across handlers/tasks is required. Avoid
global mutable state, service locators, and a generic DI container.

Use Rust ownership and borrowing to make lifetimes and mutation explicit.
Prefer owned values where they simplify async boundaries. Use `Arc` only for
genuinely shared ownership, and avoid unnecessary cloning of pools or state
that already has cheap shared semantics.

# 11. Proposed Directory Structure

```text
.
├── Cargo.toml
├── Cargo.lock
├── Dockerfile
├── docker-compose.yml
├── README.md
├── config/
│   ├── development.example.env
│   └── test.example.env
├── migrations/
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── domain/
│   │   ├── model/
│   │   ├── repository/
│   │   ├── service/
│   │   └── error/
│   ├── application/
│   │   ├── dto/
│   │   ├── usecase/
│   │   └── port/
│   ├── infrastructure/
│   │   ├── database/postgres/
│   │   ├── repository/
│   │   └── logging/
│   └── presentation/http/
│       ├── handler/
│       ├── dto/
│       ├── mapper/
│       └── middleware/
└── tests/
    ├── database/
    └── http/
```

This is a starting point, not a requirement to create empty modules. The
structure may evolve as domain boundaries become clear.

# 12. HTTP API

The API must expose:

```http
GET /health
```

Response:

```json
{"status":"UP"}
```

Status: `200 OK`. A readiness check that verifies PostgreSQL may be added
separately; liveness should not fail solely because an optional dependency is
temporarily unavailable.

Serve the generated OpenAPI document at `/api-docs/openapi.json` and the
interactive Swagger UI at `/swagger-ui`. The specification must document the
health endpoint, full User CRUD, request/response schemas, and documented
error responses. Keep the OpenAPI annotations alongside the HTTP handlers and
DTOs so they evolve with the API.

The reference feature may include:

```http
POST /api/v1/users
GET  /api/v1/users/{id}
```

`User` is a reference implementation and may later be replaced by the actual
project domain.

# 13. HTTP Conventions

Use resource-oriented URLs:

```text
GET    /api/v1/users
GET    /api/v1/users/{id}
POST   /api/v1/users
PUT    /api/v1/users/{id}
DELETE /api/v1/users/{id}
DELETE /api/v1/users
```

`DELETE /api/v1/users` removes every user and returns `204`, including when
the collection is already empty. This destructive operation requires
authorization before production use. Avoid action-oriented routes such as
`POST /api/createUser`. Use appropriate status codes, including `200`, `201`,
`204`, `400`, `404`, `409`, and `500`.

# 14. Serialization and Errors

Use Serde for transport serialization and deserialization. HTTP DTOs should
remain distinct from domain types when that separation improves validation or
prevents transport concerns from leaking inward. Do not derive serialization
traits on domain types by default without a reason.

Expose a consistent error response:

```json
{
  "code": "USER_NOT_FOUND",
  "message": "User was not found",
  "traceId": "..."
}
```

Required fields are `code`, `message`, and `traceId`; `details` and
`fieldErrors` are optional. Internal errors must not expose SQL, PostgreSQL
messages, filesystem paths, or stack traces. Map typed application errors to
HTTP status and safe response bodies at the presentation boundary.

# 15. Logging and Observability

Use `tracing` spans and structured events, with `tracing-subscriber` configured
at startup. Include useful request fields such as method, path, status, and
duration, plus a request/trace identifier where available. Avoid logging
sensitive payloads or secrets. `tracing` does not by itself provide a
distributed tracing backend; OpenTelemetry integration is a future decision.

# 16. Testing Strategy

Use Rust's built-in unit and integration test harness (`#[test]`, `#[tokio::test]`).
Unit tests should cover domain behavior and use cases. Test application logic
through repository ports with small fakes or mocks when they improve clarity.
Integration tests should cover SeaORM repositories and PostgreSQL migrations
against a real PostgreSQL instance, preferably started with Docker Compose.
HTTP integration tests may run the Axum router in-process and exercise actual
request/response behavior.

Tests should be deterministic, isolated, and avoid depending on external
services unless explicitly categorized as integration tests. Follow the
pattern `Method_Scenario_ExpectedResult` or readable behavior-based names.

# 17. Configuration and Secrets

Configuration must not be hard-coded in application logic. Expected
environment variables include:

```text
HTTP_HOST
HTTP_PORT
DATABASE_URL
DATABASE_MAX_CONNECTIONS
LOG_LEVEL
```

Secrets are supplied externally and must not be committed. Validate required
configuration at startup and fail with a clear, safe diagnostic if invalid.

# 18. Docker and Development Environment

Docker Compose provides PostgreSQL for local development and integration
tests. Its local credentials are for development only; `POSTGRES_PASSWORD`
may override the local password. The database is reachable at
`localhost:5432` from the host and `db:5432` from containers on the Compose
network.

Use Docker Compose for PostgreSQL and run the Rust application and Cargo
commands directly in the local development environment. Production image
construction and deployment remain separate from this development setup.

# 19. Development Workflow

Start PostgreSQL with `docker compose up -d db`. Run the application and checks
with the local Rust toolchain:

```bash
cargo build
cargo test
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

The development environment should provide a stable Rust toolchain, Cargo,
rustfmt, Clippy, and Docker Compose. Document SeaORM migration commands in the
README.

# 20. Quality and Reliability

Use `cargo fmt` for deterministic formatting and Clippy for static analysis.
Treat project Clippy warnings as errors in CI. Prefer compiler warnings and
idiomatic Rust ownership/error handling over unsafe code. Unsafe Rust is not
expected in application code and requires a documented, narrowly scoped
reason.

Use typed errors and explicit `Result` propagation. Do not use panics for
expected runtime failures. Avoid `unwrap`/`expect` in request handling and
infrastructure paths; they may be appropriate in tests or for invariants
validated during startup.

# 21. Initial Database Schema

If the reference `User` feature is implemented, the first SeaORM migration may
create:

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY,
    name VARCHAR(200) NOT NULL,
    email VARCHAR(320) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);
```

Schema changes must be versioned in `migrations/`. Do not rely on implicit
schema creation in production.

# 22. Initial Feature

The first complete vertical slice should preferably be `Create User`:

```text
POST /api/v1/users -> Axum handler -> CreateUser use case
                   -> UserRepository port -> SeaORM adapter -> PostgreSQL
```

Request:

```json
{"name":"John Doe","email":"john@example.com"}
```

Response (`201 Created`):

```json
{"id":"uuid","name":"John Doe","email":"john@example.com"}
```

# 23. Implementation Phases

## Phase 1 — Project bootstrap

Create the Cargo project, configure the Rust edition and dependencies, establish
the initial module structure, and add Docker Compose support for PostgreSQL.

Acceptance: `cargo build` and `cargo test` succeed.

## Phase 2 — Minimal HTTP server

Implement `GET /health` with the specified status and JSON body. Configure
startup logging and graceful shutdown where practical.

## Phase 3 — PostgreSQL

Add PostgreSQL configuration, SeaORM `DatabaseConnection`, checked-in migrations, and
database health/readiness verification.

## Phase 4 — First vertical slice

Implement the User model, application use case, repository port, SeaORM
repository, HTTP DTO/handler, and `POST /api/v1/users`.

## Phase 5 — Testing

Add unit tests, repository integration tests, migration coverage, and HTTP
integration tests.

## Phase 6 — Quality tooling

Make rustfmt and Clippy checks reproducible and enable them in CI when CI is
introduced.

# 24. Definition of Done

A feature is complete when it builds without project warnings, relevant unit
and integration tests pass, formatting and Clippy checks pass, public APIs are
documented where needed, HTTP errors follow the standard format, business
logic remains outside handlers, and SeaORM infrastructure types do not leak
into domain/application contracts.

# 25. Design Principles

Prefer explicit over implicit, composition over inheritance, ownership and
borrowing over unnecessary shared mutability, typed errors over stringly
typed control flow, constructor/composition-root wiring over service locators,
and simple abstractions over generic ones. Avoid global mutable state,
unnecessary `Arc`, deep indirection, framework types in the domain, and
premature abstractions.

Use named fields for structs that hold application state, dependencies, or
multiple values with distinct meanings. Reserve tuple structs for intentional
newtypes or cases where positional meaning is part of the type's purpose.

# 26. Codex Implementation Guidelines

1. Prefer incremental changes.
2. Keep each architectural layer independently understandable.
3. Add dependencies only for clear requirements.
4. Keep business logic out of Axum handlers.
5. Wire dependencies explicitly in the composition root.
6. Use Rust ownership, borrowing, and typed errors deliberately.
7. Add tests alongside application behavior.
8. Maintain the dependency direction between layers.
9. Update this specification when architectural decisions change.
10. Record major architectural choices before implementing large changes.

# 27. Initial Codex Tasks

```text
1. Bootstrap a Rust 2024 Cargo application.
2. Add Axum, Tokio, Serde, SeaORM/PostgreSQL, and tracing.
3. Create the initial domain, application, infrastructure, and HTTP modules.
4. Create the application entry point and explicit dependency wiring.
5. Implement GET /health returning {"status":"UP"}.
6. Add SeaORM migration and PostgreSQL development infrastructure.
7. Add unit and integration test coverage.
8. Document the local Cargo workflow and Docker Compose PostgreSQL service.
9. Configure rustfmt and Clippy checks.
10. Verify the application builds and tests successfully.
11. Do not implement domain features beyond what validates the architecture.
```

# 28. Future Decisions

Decide these when requirements justify them:

```text
Authentication and authorization
OpenTelemetry / distributed tracing
Metrics / Prometheus
Redis and Kafka
Rate limiting and idempotency
Pagination conventions and API versioning
Graceful shutdown policy
Secrets management
Production deployment
SeaORM entity and migration workflow
```

# 29. Architecture Decision Log

Record significant choices under `docs/adr/`, using a structure such as:

```text
docs/adr/
├── 0001-use-axum-tokio.md
├── 0002-use-clean-architecture.md
├── 0004-use-postgresql-and-sqlx.md
└── 0009-use-seaorm.md
```

Each ADR should describe context, decision, consequences, and alternatives
considered.

# 30. Current Architectural Decisions

## ADR-001 — Axum and Tokio

Decision: use Axum for HTTP and Tokio as the asynchronous runtime.

Rationale: they provide a well-supported Rust HTTP and async foundation while
allowing handlers and runtime concerns to remain at the system boundary.

## ADR-002 — Layered architecture

Decision: keep domain and application logic independent from Axum and SeaORM.

Rationale: this keeps business behavior testable and external adapters
replaceable.

## ADR-003 — Cargo

Decision: use Cargo for builds, dependency management, tests, formatting, and
linting.

## ADR-004 — PostgreSQL and SQLx

Initial decision: use PostgreSQL as the primary relational database and SQLx
as its Rust client, pool, query, and migration integration.

Status: superseded by ADR-009. SeaORM continues to use SQLx internally for its
PostgreSQL driver, but application code no longer uses SQLx directly.

## ADR-005 — Serde

Decision: use Serde for JSON request/response serialization at the HTTP
boundary.

## ADR-006 — tracing

Decision: use `tracing` and `tracing-subscriber` for structured application
logging and instrumentation.

## ADR-007 — Local Cargo workflow with Docker Compose

Decision: run Rust/Cargo tooling directly in the local development environment
and provide PostgreSQL through Docker Compose.

Rationale: keep the experiment's setup small while isolating the database
service and its persisted data in Docker.

## ADR-008 — OpenAPI and Swagger UI

Decision: generate the OpenAPI document from Rust types and Axum handler
annotations with Utoipa, and serve it with Swagger UI using the vendored UI
assets.

Rationale: provide interactive, versioned API documentation without relying
on a runtime CDN or a separate manually maintained specification.

## ADR-009 — SeaORM for PostgreSQL persistence

Decision: use SeaORM for the PostgreSQL connection, repository queries, entity
mapping, and versioned migrations. Keep the migration definitions in the
application crate because the current deployment runs them through the
application startup path; do not add a separate migration crate or CLI yet.

Rationale: the application is a single small service with one repository and
already applies migrations from its composition root. SeaORM removes direct
SQLx usage while keeping the repository port and HTTP contract unchanged.

Consequences: SeaORM owns a separate migration history table. A database
previously initialized by SQLx requires manual preparation or a new empty
database before this version starts; startup never resets or drops a database.
SeaORM continues to use SQLx internally for its PostgreSQL driver.

# 31. Guiding Constraint

The project should remain a **Rust application that happens to expose HTTP**.
Framework convenience must not override clear ownership, dependency direction,
typed error handling, testability, or idiomatic Rust.
