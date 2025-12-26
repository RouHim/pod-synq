# PodSynq Agent Guidelines

This file provides guidance for agentic coding agents working on the PodSynq codebase.

---

## Commands

### Build & Run
- `cargo build` - Build the project
- `cargo build --release` - Build optimized release binary
- `cargo run` - Run the development server
- `cargo check` - Quick compile check without building

### Testing
- `cargo test` - Run all tests
- `cargo test <test_name>` - Run a specific test
- `cargo test -- --test-threads=1` - Run tests single-threaded (useful for tests sharing DB)
- `cargo test -- --nocapture` - Show test output

### Linting & Formatting
- `cargo fmt` - Format code according to Rust style
- `cargo clippy` - Run Clippy linter
- `cargo clippy -- -D warnings` - Treat warnings as errors

### Database
- The SQLite database file is at `./pod-synq.db` (default)
- Migrations are in `migrations/001_initial.sql`
- To reset database: `rm pod-synq.db`

---

## Code Style Guidelines

### Architecture
- **Layered Architecture**: Models → Repository → Service → Handler → Route
- **Models** (`src/models/`): Data structures with Serialize/Deserialize
- **Repository** (`src/repository/`): Database access, raw SQL queries
- **Services** (`src/services/`): Business logic, validation
- **Handlers** (`src/handlers/`): HTTP request/response handling
- **Middleware** (`src/middleware/`): Auth, request context
- **State** (`src/state.rs`): Application state with Arc-wrapped services

### Module Structure
- Each domain (user, device, subscription, etc.) has files in models/, repository/, services/
- `mod.rs` files re-export public types with `pub use`
- Keep modules focused - one responsibility per file
- Prefer separate files over nested modules

### Naming Conventions
- **Types**: PascalCase (e.g., `UserService`, `CreateUser`)
- **Functions/Variables**: snake_case (e.g., `get_user`, `user_id`)
- **Constants/Env Vars**: SCREAMING_SNAKE_CASE (e.g., `PODSYNQ_PORT`)
- **Table names**: snake_case (e.g., `users`, `episode_actions`)
- **Private fields**: Prefix with underscore if unused: `_username`

### Error Handling
- Custom errors in `src/error.rs` using `thiserror::Error`
- Use `AppResult<T>` type alias: `pub type AppResult<T> = Result<T, AppError>`
- Return specific error variants (e.g., `AppError::UserNotFound`, `AppError::Authentication`)
- Implement `IntoResponse` for AppError to return proper HTTP status codes
- Use `anyhow::Result` in `main.rs` only
- Never use `unwrap()` or `expect()` in production code - use `?` operator

### Database (SQLx)
- Use parameterized queries with `.bind()` to prevent SQL injection
- Use `fetch_one()`, `fetch_optional()`, `fetch_all()` as appropriate
- Use transactions with `pool.begin().await?` for multi-step operations
- Convert SQLite BOOLEAN to INTEGER with `CAST(is_admin AS INTEGER)`
- Use `strftime('%s', 'now')` for Unix timestamps

### Async & Concurrency
- All database and network operations are async
- Services are wrapped in `Arc` for sharing across requests
- Use `#[tokio::main]` for async main function
- Use `async fn` for handlers, repositories, and services

### Logging
- Use `tracing` crate, not `println!` or `log!`
- Use `tracing::info!`, `tracing::warn!`, `tracing::error!` as appropriate
- Include contextual information in logs
- Set log level via `RUST_LOG` environment variable

### Password Security
- Hash passwords with Argon2 via `argon2` crate
- Use `UserService::hash_password()` and `UserService::verify_password()`
- Never store plain-text passwords
- Compare hash with `PasswordVerifier::verify_password()`

### Configuration
- Read config from environment variables in `src/config.rs`
- Config vars: `PODSYNQ_PORT`, `PODSYNQ_DB_PATH`, `PODSYNQ_ADMIN_USERNAME`, `PODSYNQ_ADMIN_PASSWORD`
- Provide sensible defaults in `Config::from_env()`
- Validate config in `Config::validate()`

### Technical Requirements

- External crates are allowed, but keep them as low as possible
- Prefer standard Rust libraries and built-in features to minimize external package usage.
- Evaluate trade-offs before adding any third-party crate.
- When using external crates, make sure to use the very latest stable versions.
- All static files needs to be embedded into the binary
- Must compile and run without errors
- Handle user interactions gracefully
- Implement proper error handling and validation
- Use appropriate Rust idioms and patterns
- Logging: prefer `tracing`/`tracing_subscriber` with contextual spans instead of `println!`.
- Error handling: avoid `unwrap`/`expect` in non-test code; surface actionable errors to the UI.
- Structure code into small, focused rust files without using rust modules
- Each file should encapsulate a single responsibility or closely related functionalities.
- Promote reusability and ease of testing by isolating components.
- Follow the SOLID object-oriented design principles to ensure maintainable and extensible code.
- Emphasize single responsibility, open-closed, Liskov substitution, interface segregation, and dependency inversion
  where applicable.
- Use descriptive names and avoid clever tricks or shortcuts that hinder comprehensibility.
- YAGNI - You Aren't Gonna Need It: Avoid adding functionality until it is necessary.
- Don't write unused code for future features.
- Always run code formatters (`cargo fmt`) and linters (`cargo clippy`) when finishing a task.
- Maintain consistent code style across the project to improve readability and reduce friction in reviews.
- Always use RustTLS for any TLS connections, no OpenSSL.

## Testing Practices

### Test-Driven Development (TDD)

- Prefer write tests before writing the functionality.
- Use tests to drive design decisions and ensure robust feature implementation.

### Behavior-Driven Development (BDD)

- Write tests in a BDD style, focusing on the expected behavior and outcomes.
- Structure tests to clearly state scenarios, actions, and expected results to improve communication and documentation.
