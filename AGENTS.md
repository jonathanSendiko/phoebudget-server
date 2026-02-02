# Repository Guidelines

## Project Structure & Module Organization
- `src/` is the Rust application code. Core modules include `handlers.rs` (HTTP handlers), `services.rs`, `repository.rs`, and `auth.rs`.
- `src/bin/` contains the binaries: `server.rs` (API), `scheduler.rs`, and `worker.rs`.
- `migrations/` holds SQLx migration files; follow the timestamp prefix pattern in filenames.
- `docs/` and `NEW_FEATURE_API.md` contain product notes/specs.
- `nginx/` and `docker-compose*.yml` are deployment/runtime configs.

## Build, Test, and Development Commands
- `cargo build` builds the workspace binaries.
- `cargo run --bin server` runs the API locally.
- `cargo run --bin scheduler` and `cargo run --bin worker` run background jobs.
- `cargo test` runs the Rust test suite.
- `docker compose up --build` starts local Postgres/Redis plus the API stack.

## Coding Style & Naming Conventions
- Follow standard Rust formatting and idioms; keep modules small and focused.
- Use `snake_case` for functions/variables and `CamelCase` for types.
- Keep handler names aligned with routes (e.g., `get_portfolio`, `create_goal`).
- Migrations should be additive and backwards-aware; avoid editing old migration files.

## Testing Guidelines
- Tests live alongside modules (e.g., `src/portfolio/tests.rs`); add new tests close to the code they cover.
- Prefer unit tests for pure logic and integration tests for repository/DB behavior.
- Run `cargo test` before opening a PR; add tests for new endpoints or migrations.

## Commit & Pull Request Guidelines
- Commit messages in history are short, imperative, and plain (e.g., “add sqlx”, “update DEPLOY.md”).
- Keep commits focused; include migration files in the same commit as code changes that depend on them.
- PRs should describe the change, list tests run, and call out any DB or config changes.

## Configuration & Security Notes
- Local services are defined in `docker-compose.yml`; default ports are `3000`, `5433`, and `6379`.
- Secrets/config are expected via environment variables (see `docker-compose.yml` for examples).
- For production, follow `DEPLOY.md` and use `.env.prod` with `setup.sh`.
