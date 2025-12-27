# E2E Tests for PodSynq

This directory contains end-to-end tests for PodSynq to verify compliance with the gpodder.net API specification.

## Requirements

The test suite requires the following CLI tools:

- `curl` - For making HTTP requests
- `jq` - For JSON parsing and validation
- `base64` - For encoding (usually preinstalled)
- `date` - For timestamp generation (usually preinstalled)

### Installation

**Debian/Ubuntu:**
```bash
sudo apt-get install curl jq coreutils
```

**Fedora/RHEL:**
```bash
sudo dnf install curl jq coreutils
```

**macOS:**
```bash
brew install curl jq coreutils
```

**Arch Linux:**
```bash
sudo pacman -S curl jq coreutils
```

## Running Tests

### 1. Start the PodSynq server

First, ensure the server is running:

```bash
cargo run
```

Or in a separate terminal:

```bash
cargo build --release
./target/release/pod-synq
```

### 2. Run the E2E tests

```bash
./tests/e2e.sh
```

### 3. Run against a different URL

Set the `PODSYNQ_URL` environment variable:

```bash
PODSYNQ_URL=http://localhost:8080 ./tests/e2e.sh
```

## Test Coverage

The test suite validates compliance with the gpodder.net API v2.11 specification:

### Authentication API
- ✅ Login with HTTP Basic Auth
- ✅ Login failure with invalid credentials
- ✅ Logout

### Device API
- ✅ Create device with metadata
- ✅ List all devices
- ✅ Update device metadata
- ✅ Get device updates/changes

### Subscriptions API (Simple)
- ✅ Upload subscriptions (TXT format)
- ✅ Get subscriptions (TXT format)
- ✅ Get subscriptions (JSON format)
- ✅ Get all subscriptions across devices

### Subscriptions API (Advanced)
- ✅ Upload subscription changes (add/remove)
- ✅ Get subscription changes with timestamp
- ✅ Incremental sync with `since` parameter

### Episode Actions API
- ✅ Upload episode actions (download, play, delete, new)
- ✅ Get episode actions with timestamp
- ✅ Different action types validation

### Settings API
- ✅ Save account-scoped settings
- ✅ Get account-scoped settings
- ✅ Save device-scoped settings

### Multi-Device Scenarios
- ✅ Multi-device subscription sync
- ✅ Incremental sync with timestamps
- ✅ Cross-device subscription merging

## Test User

The tests use a preconfigured test user:

- **Username:** `testuser`
- **Password:** `testpass123`

**Note:** Make sure this user exists in your PodSynq database before running tests. You can create it via the admin interface or by setting environment variables:

```bash
PODSYNQ_ADMIN_USERNAME=testuser PODSYNQ_ADMIN_PASSWORD=testpass123 cargo run
```

## Output

The test script provides colored output:

- 🔵 **Blue (ℹ)** - Informational messages
- ✅ **Green (✓)** - Passed tests
- ❌ **Red (✗)** - Failed tests
- ⚠️ **Yellow (⚠)** - Warnings

Example output:
```
========================================
   PodSynq E2E Test Suite
========================================
Base URL: http://localhost:3000
Test User: testuser
========================================

ℹ Testing Authentication API - Login
✓ Login with valid credentials (HTTP 200)
...
========================================
          TEST SUMMARY
========================================
Total tests:  42
Passed:       42
Failed:       0
========================================
All tests passed! ✓
```

## Exit Codes

- `0` - All tests passed
- `1` - One or more tests failed or dependencies missing

## Troubleshooting

### Server not starting

If tests fail with "Server did not start in time":

1. Make sure PodSynq is running on the correct port
2. Check if the port is already in use: `lsof -i :3000`
3. Verify the server is accessible: `curl http://localhost:3000/`

### Missing dependencies

If you see "Missing required tools", install the required packages as shown above.

### Test user not found

Create the test user before running tests, or modify the `TEST_USER` and `TEST_PASS` variables in `e2e.sh` to match your existing user credentials.

## API Reference

Tests are based on the official gpodder.net API documentation:
- https://gpoddernet.readthedocs.io/en/latest/api/
- OpenAPI spec: https://raw.githubusercontent.com/gpodder/mygpo/master/mygpo/api/openapi.yaml
