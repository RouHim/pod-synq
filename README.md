# PodSynq

A podcast synchronization server that implements the gpodder.net API specification.

## Features

- User authentication with Argon2 password hashing
- Device management and synchronization
- Subscription management
- Episode tracking and playback progress
- Podcast directory integration
- REST API compatible with gpodder.net clients

## Architecture

The application follows a layered architecture:

- **Models**: Data structures with serialization support
- **Repository**: Database access layer using SQLx with SQLite
- **Services**: Business logic and validation
- **Handlers**: HTTP request/response handling with Warp
- **Middleware**: Authentication and request context

## Installation

### From source

```bash
cargo build --release
```

The binary will be available at `target/release/pod-synq`.

### Cross-platform binaries

Pre-built binaries are available in the [releases](https://github.com/rouven/pod-synq/releases) section.

## Configuration

The application is configured via environment variables:

- `PODSYNQ_PORT` - Server port (default: 8000)
- `PODSYNQ_DB_PATH` - Database file path (default: ./pod-synq.db)
- `PODSYNQ_ADMIN_USERNAME` - Admin username (default: admin)
- `PODSYNQ_ADMIN_PASSWORD` - Admin password (default: admin)

## Usage

Run the server:

```bash
cargo run
```

Or use the release binary:

```bash
./target/release/pod-synq
```

## Development

### Build

```bash
cargo build
```

### Run tests

**Unit tests:**
```bash
cargo test
```

**E2E tests:**
```bash
# Start the server first
cargo run

# In another terminal, run E2E tests
./tests/e2e.sh
```

See [tests/README.md](tests/README.md) for more details on E2E testing.

### Format code

```bash
cargo fmt
```

### Run linter

```bash
cargo clippy
```

## API

The server implements the gpodder.net API specification. See the [API documentation](docs/api.md) for details.

## License

MIT
