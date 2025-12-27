#!/usr/bin/env bash

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
BASE_URL="${PODSYNQ_URL:-http://localhost:3000}"
TEST_USER="testuser"
TEST_PASS="testpass123"
TEST_DEVICE="test-device-001"
TEST_DEVICE2="test-device-002"

# Test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Check required tools
check_dependencies() {
    local missing=()
    
    command -v curl >/dev/null 2>&1 || missing+=("curl")
    command -v jq >/dev/null 2>&1 || missing+=("jq")
    command -v base64 >/dev/null 2>&1 || missing+=("base64")
    command -v date >/dev/null 2>&1 || missing+=("date")
    
    if [ ${#missing[@]} -ne 0 ]; then
        echo -e "${RED}Error: Missing required tools: ${missing[*]}${NC}"
        echo "Please install them first:"
        echo "  - Debian/Ubuntu: sudo apt-get install curl jq coreutils"
        echo "  - Fedora/RHEL: sudo dnf install curl jq coreutils"
        echo "  - macOS: brew install curl jq coreutils"
        exit 1
    fi
}

# Utility functions
log_info() {
    echo -e "${BLUE}ℹ${NC} $*"
}

log_success() {
    echo -e "${GREEN}✓${NC} $*"
}

log_error() {
    echo -e "${RED}✗${NC} $*"
}

log_warning() {
    echo -e "${YELLOW}⚠${NC} $*"
}

# Test assertion helpers
assert_http_code() {
    local actual="$1"
    local expected="$2"
    local test_name="$3"
    
    TESTS_RUN=$((TESTS_RUN + 1))
    
    if [ "$actual" = "$expected" ]; then
        TESTS_PASSED=$((TESTS_PASSED + 1))
        log_success "$test_name (HTTP $actual)"
        return 0
    else
        TESTS_FAILED=$((TESTS_FAILED + 1))
        log_error "$test_name (expected HTTP $expected, got $actual)"
        return 1
    fi
}

# HTTP helper
http_request() {
    local method="$1"
    local path="$2"
    local auth="${3:-}"
    local data="${4:-}"
    local content_type="${5:-application/json}"
    
    local curl_args=(-s -w "\n%{http_code}" -X "$method")
    
    if [ -n "$auth" ]; then
        curl_args+=(-u "$auth")
    fi
    
    if [ -n "$data" ]; then
        curl_args+=(-H "Content-Type: $content_type" -d "$data")
    fi
    
    curl "${curl_args[@]}" "$BASE_URL$path"
}

# Wait for server to be ready
wait_for_server() {
    log_info "Waiting for server at $BASE_URL..."
    local max_attempts=30
    local attempt=0
    
    while [ $attempt -lt $max_attempts ]; do
        if curl -s -f "$BASE_URL/" >/dev/null 2>&1; then
            log_success "Server is ready"
            return 0
        fi
        attempt=$((attempt + 1))
        sleep 1
    done
    
    log_error "Server did not start in time"
    exit 1
}

# Test: Authentication API
test_auth_login() {
    echo
    log_info "Testing Authentication API - Login"
    
    local response
    response=$(http_request GET "/api/2/auth/$TEST_USER/login.json" "$TEST_USER:$TEST_PASS")
    local status=$(echo "$response" | tail -n 1)
    
    assert_http_code "$status" "200" "Login with valid credentials"
}

test_auth_login_invalid() {
    log_info "Testing Authentication API - Invalid Login"
    
    local response
    response=$(http_request GET "/api/2/auth/$TEST_USER/login.json" "$TEST_USER:wrongpass")
    local status=$(echo "$response" | tail -n 1)
    
    assert_http_code "$status" "401" "Login with invalid credentials"
}

test_auth_logout() {
    log_info "Testing Authentication API - Logout"
    
    local response
    response=$(http_request POST "/api/2/auth/$TEST_USER/logout.json" "$TEST_USER:$TEST_PASS" "{}")
    local status=$(echo "$response" | tail -n 1)
    
    assert_http_code "$status" "200" "Logout"
}

# Test: Device API
test_device_create() {
    echo
    log_info "Testing Device API - Create Device"
    
    local device_data='{"caption":"Test Laptop","type":"laptop"}'
    local response
    response=$(http_request POST "/api/2/devices/$TEST_USER/$TEST_DEVICE.json" "$TEST_USER:$TEST_PASS" "$device_data")
    local status=$(echo "$response" | tail -n 1)
    
    assert_http_code "$status" "200" "Create device"
}

test_device_list() {
    log_info "Testing Device API - List Devices"
    
    local response
    response=$(http_request GET "/api/2/devices/$TEST_USER/.json" "$TEST_USER:$TEST_PASS")
    local body=$(echo "$response" | head -n -1)
    local status=$(echo "$response" | tail -n 1)
    
    assert_http_code "$status" "200" "List devices"
    
    # Check that our device is in the list
    local device_count
    device_count=$(echo "$body" | jq '[.[] | select(.id == "'"$TEST_DEVICE"'")] | length')
    
    TESTS_RUN=$((TESTS_RUN + 1))
    if [ "$device_count" -ge 1 ]; then
        TESTS_PASSED=$((TESTS_PASSED + 1))
        log_success "Device found in list"
    else
        TESTS_FAILED=$((TESTS_FAILED + 1))
        log_error "Device not found in list"
    fi
}

test_device_update() {
    log_info "Testing Device API - Update Device"
    
    local device_data='{"caption":"Updated Laptop","type":"desktop"}'
    local response
    response=$(http_request POST "/api/2/devices/$TEST_USER/$TEST_DEVICE.json" "$TEST_USER:$TEST_PASS" "$device_data")
    local status=$(echo "$response" | tail -n 1)
    
    assert_http_code "$status" "200" "Update device metadata"
}

# Test: Subscriptions API - Simple
test_subscriptions_simple_upload_txt() {
    echo
    log_info "Testing Subscriptions API (Simple) - Upload TXT"
    
    local subs="http://example.com/feed1.rss
http://example.org/feed2.xml
http://example.net/podcast.rss"
    
    local response
    response=$(http_request PUT "/subscriptions/$TEST_USER/$TEST_DEVICE/txt" "$TEST_USER:$TEST_PASS" "$subs" "text/plain")
    local status=$(echo "$response" | tail -n 1)
    
    assert_http_code "$status" "200" "Upload subscriptions (TXT format)"
}

test_subscriptions_simple_get_txt() {
    log_info "Testing Subscriptions API (Simple) - Get TXT"
    
    local response
    response=$(http_request GET "/subscriptions/$TEST_USER/$TEST_DEVICE/txt" "$TEST_USER:$TEST_PASS")
    local body=$(echo "$response" | head -n -1)
    local status=$(echo "$response" | tail -n 1)
    
    assert_http_code "$status" "200" "Get subscriptions (TXT format)"
    
    # Check that subscriptions are present
    TESTS_RUN=$((TESTS_RUN + 1))
    if echo "$body" | grep -q "example.com/feed1.rss"; then
        TESTS_PASSED=$((TESTS_PASSED + 1))
        log_success "Subscriptions contain expected feed"
    else
        TESTS_FAILED=$((TESTS_FAILED + 1))
        log_error "Subscriptions missing expected feed"
    fi
}

test_subscriptions_simple_get_json() {
    log_info "Testing Subscriptions API (Simple) - Get JSON"
    
    local response
    response=$(http_request GET "/subscriptions/$TEST_USER/$TEST_DEVICE/json" "$TEST_USER:$TEST_PASS")
    local body=$(echo "$response" | head -n -1)
    local status=$(echo "$response" | tail -n 1)
    
    assert_http_code "$status" "200" "Get subscriptions (JSON format)"
    
    # Validate it's valid JSON
    TESTS_RUN=$((TESTS_RUN + 1))
    if echo "$body" | jq empty 2>/dev/null; then
        TESTS_PASSED=$((TESTS_PASSED + 1))
        log_success "Response is valid JSON"
    else
        TESTS_FAILED=$((TESTS_FAILED + 1))
        log_error "Response is not valid JSON"
    fi
}

test_subscriptions_simple_all_devices() {
    log_info "Testing Subscriptions API (Simple) - Get All Devices"
    
    local response
    response=$(http_request GET "/subscriptions/$TEST_USER/txt" "$TEST_USER:$TEST_PASS")
    local status=$(echo "$response" | tail -n 1)
    
    assert_http_code "$status" "200" "Get all subscriptions across devices"
}

# Test: Subscriptions API - Advanced
test_subscriptions_advanced_upload() {
    echo
    log_info "Testing Subscriptions API (Advanced) - Upload Changes"
    
    local changes='{"add":["http://newpodcast.com/feed.rss"],"remove":["http://example.net/podcast.rss"]}'
    
    local response
    response=$(http_request POST "/api/2/subscriptions/$TEST_USER/$TEST_DEVICE.json" "$TEST_USER:$TEST_PASS" "$changes")
    local body=$(echo "$response" | head -n -1)
    local status=$(echo "$response" | tail -n 1)
    
    assert_http_code "$status" "200" "Upload subscription changes"
    
    # Check response contains timestamp
    TESTS_RUN=$((TESTS_RUN + 1))
    if echo "$body" | jq -e '.timestamp' >/dev/null 2>&1; then
        TESTS_PASSED=$((TESTS_PASSED + 1))
        log_success "Response contains timestamp"
    else
        TESTS_FAILED=$((TESTS_FAILED + 1))
        log_error "Response missing timestamp"
    fi
}

test_subscriptions_advanced_get() {
    log_info "Testing Subscriptions API (Advanced) - Get Changes"
    
    local response
    response=$(http_request GET "/api/2/subscriptions/$TEST_USER/$TEST_DEVICE.json?since=0" "$TEST_USER:$TEST_PASS")
    local body=$(echo "$response" | head -n -1)
    local status=$(echo "$response" | tail -n 1)
    
    assert_http_code "$status" "200" "Get subscription changes"
    
    # Validate response structure
    TESTS_RUN=$((TESTS_RUN + 1))
    if echo "$body" | jq -e '.add' >/dev/null 2>&1 && echo "$body" | jq -e '.remove' >/dev/null 2>&1; then
        TESTS_PASSED=$((TESTS_PASSED + 1))
        log_success "Response has correct structure (add/remove)"
    else
        TESTS_FAILED=$((TESTS_FAILED + 1))
        log_error "Response missing add/remove arrays"
    fi
}

# Test: Episode Actions API
test_episode_actions_upload() {
    echo
    log_info "Testing Episode Actions API - Upload Actions"
    
    local timestamp
    timestamp=$(date -u +"%Y-%m-%dT%H:%M:%S")
    
    local actions='[{"podcast":"http://example.com/feed1.rss","episode":"http://example.com/episode1.mp3","device":"'"$TEST_DEVICE"'","action":"download","timestamp":"'"$timestamp"'"},{"podcast":"http://example.com/feed1.rss","episode":"http://example.com/episode1.mp3","action":"play","started":0,"position":120,"total":500,"timestamp":"'"$timestamp"'"}]'
    
    local response
    response=$(http_request POST "/api/2/episodes/$TEST_USER.json" "$TEST_USER:$TEST_PASS" "$actions")
    local body=$(echo "$response" | head -n -1)
    local status=$(echo "$response" | tail -n 1)
    
    assert_http_code "$status" "200" "Upload episode actions"
    
    # Check response structure
    TESTS_RUN=$((TESTS_RUN + 1))
    if echo "$body" | jq -e '.timestamp' >/dev/null 2>&1; then
        TESTS_PASSED=$((TESTS_PASSED + 1))
        log_success "Response contains timestamp"
    else
        TESTS_FAILED=$((TESTS_FAILED + 1))
        log_error "Response missing timestamp"
    fi
}

test_episode_actions_get() {
    log_info "Testing Episode Actions API - Get Actions"
    
    local response
    response=$(http_request GET "/api/2/episodes/$TEST_USER.json?since=0" "$TEST_USER:$TEST_PASS")
    local body=$(echo "$response" | head -n -1)
    local status=$(echo "$response" | tail -n 1)
    
    assert_http_code "$status" "200" "Get episode actions"
    
    # Check response is array
    TESTS_RUN=$((TESTS_RUN + 1))
    if echo "$body" | jq -e '.actions | type == "array"' >/dev/null 2>&1; then
        TESTS_PASSED=$((TESTS_PASSED + 1))
        log_success "Response contains actions array"
    else
        TESTS_FAILED=$((TESTS_FAILED + 1))
        log_error "Response missing actions array"
    fi
}

test_episode_actions_types() {
    log_info "Testing Episode Actions API - Different Action Types"
    
    local timestamp
    timestamp=$(date -u +"%Y-%m-%dT%H:%M:%S")
    
    # Test delete action
    local delete_action='[{"podcast":"http://example.com/feed1.rss","episode":"http://example.com/episode2.mp3","action":"delete","timestamp":"'"$timestamp"'"}]'
    
    local response
    response=$(http_request POST "/api/2/episodes/$TEST_USER.json" "$TEST_USER:$TEST_PASS" "$delete_action")
    local status=$(echo "$response" | tail -n 1)
    
    assert_http_code "$status" "200" "Upload delete action"
    
    # Test new action
    local new_action='[{"podcast":"http://example.com/feed1.rss","episode":"http://example.com/episode3.mp3","action":"new","timestamp":"'"$timestamp"'"}]'
    
    response=$(http_request POST "/api/2/episodes/$TEST_USER.json" "$TEST_USER:$TEST_PASS" "$new_action")
    status=$(echo "$response" | tail -n 1)
    
    assert_http_code "$status" "200" "Upload new action"
}

# Test: Settings API
test_settings_save() {
    echo
    log_info "Testing Settings API - Save Settings"
    
    local settings='{"key1":"value1","key2":42,"key3":true}'
    
    local response
    response=$(http_request POST "/api/2/settings/$TEST_USER/account.json" "$TEST_USER:$TEST_PASS" "$settings")
    local status=$(echo "$response" | tail -n 1)
    
    assert_http_code "$status" "200" "Save account settings"
}

test_settings_get() {
    log_info "Testing Settings API - Get Settings"
    
    local response
    response=$(http_request GET "/api/2/settings/$TEST_USER/account.json" "$TEST_USER:$TEST_PASS")
    local body=$(echo "$response" | head -n -1)
    local status=$(echo "$response" | tail -n 1)
    
    assert_http_code "$status" "200" "Get account settings"
    
    # Validate saved settings are returned
    TESTS_RUN=$((TESTS_RUN + 1))
    local key1_value
    key1_value=$(echo "$body" | jq -r '.key1')
    if [ "$key1_value" = "value1" ]; then
        TESTS_PASSED=$((TESTS_PASSED + 1))
        log_success "Settings persisted correctly"
    else
        TESTS_FAILED=$((TESTS_FAILED + 1))
        log_error "Settings not persisted correctly"
    fi
}

test_settings_device_scope() {
    log_info "Testing Settings API - Device Scope"
    
    local settings='{"device_setting":"test"}'
    
    local response
    response=$(http_request POST "/api/2/settings/$TEST_USER/$TEST_DEVICE.json" "$TEST_USER:$TEST_PASS" "$settings")
    local status=$(echo "$response" | tail -n 1)
    
    assert_http_code "$status" "200" "Save device-scoped settings"
}

# Test: Device Updates/Sync
test_device_updates() {
    echo
    log_info "Testing Device Updates API"
    
    # Create second device
    local device_data='{"caption":"Test Phone","type":"mobile"}'
    http_request POST "/api/2/devices/$TEST_USER/$TEST_DEVICE2.json" "$TEST_USER:$TEST_PASS" "$device_data" >/dev/null
    
    # Get updates
    local response
    response=$(http_request GET "/api/2/updates/$TEST_USER/$TEST_DEVICE.json?since=0" "$TEST_USER:$TEST_PASS")
    local body=$(echo "$response" | head -n -1)
    local status=$(echo "$response" | tail -n 1)
    
    assert_http_code "$status" "200" "Get device updates"
    
    # Validate response structure
    TESTS_RUN=$((TESTS_RUN + 1))
    if echo "$body" | jq -e '.add' >/dev/null 2>&1 && echo "$body" | jq -e '.timestamp' >/dev/null 2>&1; then
        TESTS_PASSED=$((TESTS_PASSED + 1))
        log_success "Device updates response has correct structure"
    else
        TESTS_FAILED=$((TESTS_FAILED + 1))
        log_error "Device updates response malformed"
    fi
}

# Test: Multi-device Sync Scenario
test_multi_device_sync() {
    echo
    log_info "Testing Multi-Device Sync Scenario"
    
    # Device 1 uploads subscriptions
    local subs1="http://device1.com/feed.rss"
    http_request PUT "/subscriptions/$TEST_USER/$TEST_DEVICE/txt" "$TEST_USER:$TEST_PASS" "$subs1" "text/plain" >/dev/null
    
    # Device 2 uploads different subscriptions
    local subs2="http://device2.com/feed.rss"
    http_request PUT "/subscriptions/$TEST_USER/$TEST_DEVICE2/txt" "$TEST_USER:$TEST_PASS" "$subs2" "text/plain" >/dev/null
    
    # Get all subscriptions across devices
    local response
    response=$(http_request GET "/subscriptions/$TEST_USER/txt" "$TEST_USER:$TEST_PASS")
    local body=$(echo "$response" | head -n -1)
    local status=$(echo "$response" | tail -n 1)
    
    assert_http_code "$status" "200" "Get merged subscriptions from all devices"
    
    # Both feeds should be present
    TESTS_RUN=$((TESTS_RUN + 1))
    if echo "$body" | grep -q "device1.com" && echo "$body" | grep -q "device2.com"; then
        TESTS_PASSED=$((TESTS_PASSED + 1))
        log_success "Multi-device subscriptions merged correctly"
    else
        TESTS_FAILED=$((TESTS_FAILED + 1))
        log_error "Multi-device subscription merge failed"
    fi
}

# Test: Incremental Sync with Timestamps
test_incremental_sync() {
    echo
    log_info "Testing Incremental Sync with Timestamps"
    
    # Get current timestamp
    local response
    response=$(http_request GET "/api/2/subscriptions/$TEST_USER/$TEST_DEVICE.json?since=0" "$TEST_USER:$TEST_PASS")
    local body=$(echo "$response" | head -n -1)
    local timestamp1
    timestamp1=$(echo "$body" | jq -r '.timestamp')
    
    # Upload new changes
    sleep 1
    local changes='{"add":["http://incremental.com/feed.rss"],"remove":[]}'
    http_request POST "/api/2/subscriptions/$TEST_USER/$TEST_DEVICE.json" "$TEST_USER:$TEST_PASS" "$changes" >/dev/null
    
    # Get updates since first timestamp
    response=$(http_request GET "/api/2/subscriptions/$TEST_USER/$TEST_DEVICE.json?since=$timestamp1" "$TEST_USER:$TEST_PASS")
    body=$(echo "$response" | head -n -1)
    local status=$(echo "$response" | tail -n 1)
    
    assert_http_code "$status" "200" "Get incremental subscription updates"
    
    # Should include the new feed
    TESTS_RUN=$((TESTS_RUN + 1))
    local new_feed_count
    new_feed_count=$(echo "$body" | jq '[.add[] | select(. == "http://incremental.com/feed.rss")] | length')
    if [ "$new_feed_count" -ge 1 ]; then
        TESTS_PASSED=$((TESTS_PASSED + 1))
        log_success "Incremental sync returns only new changes"
    else
        TESTS_FAILED=$((TESTS_FAILED + 1))
        log_error "Incremental sync did not return new changes"
    fi
}

# Print summary
print_summary() {
    echo
    echo "========================================"
    echo "          TEST SUMMARY"
    echo "========================================"
    echo -e "Total tests:  $TESTS_RUN"
    echo -e "${GREEN}Passed:       $TESTS_PASSED${NC}"
    echo -e "${RED}Failed:       $TESTS_FAILED${NC}"
    echo "========================================"
    
    if [ $TESTS_FAILED -eq 0 ]; then
        echo -e "${GREEN}All tests passed! ✓${NC}"
        return 0
    else
        echo -e "${RED}Some tests failed! ✗${NC}"
        return 1
    fi
}

# Main test runner
main() {
    echo "========================================"
    echo "   PodSynq E2E Test Suite"
    echo "========================================"
    echo "Base URL: $BASE_URL"
    echo "Test User: $TEST_USER"
    echo "========================================"
    
    # Check dependencies first
    check_dependencies
    
    # Wait for server
    wait_for_server
    
    # Run all tests
    test_auth_login
    test_auth_login_invalid
    test_auth_logout
    
    test_device_create
    test_device_list
    test_device_update
    
    test_subscriptions_simple_upload_txt
    test_subscriptions_simple_get_txt
    test_subscriptions_simple_get_json
    test_subscriptions_simple_all_devices
    
    test_subscriptions_advanced_upload
    test_subscriptions_advanced_get
    
    test_episode_actions_upload
    test_episode_actions_get
    test_episode_actions_types
    
    test_settings_save
    test_settings_get
    test_settings_device_scope
    
    test_device_updates
    test_multi_device_sync
    test_incremental_sync
    
    # Print summary
    print_summary
}

# Run main function
main "$@"
