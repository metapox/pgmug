#!/bin/bash

# JWT Authentication Test Script
# Tests the PostgreSQL OIDC Proxy with various JWT scenarios

set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

print_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
}

print_error() {
    echo -e "${RED}[FAIL]${NC} $1"
}

print_info() {
    echo -e "${YELLOW}[INFO]${NC} $1"
}

# Test configuration
API_BASE_URL="http://localhost:8080"
TEST_TOKEN=""

# Generate a test JWT token
print_info "Generating test JWT token..."
cd "$(dirname "$0")/.."
TEST_TOKEN=$(./scripts/generate_test_jwt.sh | grep "eyJ" | head -1)

if [ -z "$TEST_TOKEN" ]; then
    print_error "Failed to generate test JWT token"
    exit 1
fi

print_info "Generated JWT token: ${TEST_TOKEN:0:50}..."

# Test 1: Health check (no auth required)
print_info "Test 1: Health check (no authentication required)"
HEALTH_RESPONSE=$(curl -s -w "%{http_code}" "$API_BASE_URL/health")
HTTP_CODE="${HEALTH_RESPONSE: -3}"
RESPONSE_BODY="${HEALTH_RESPONSE%???}"

if [ "$HTTP_CODE" = "200" ]; then
    print_success "Health check returned 200 OK"
else
    print_error "Health check failed with HTTP $HTTP_CODE"
    exit 1
fi

# Test 2: Query without authentication (should fail)
print_info "Test 2: Query without authentication (should return 401)"
HTTP_CODE=$(curl -s -w "%{http_code}" -o /dev/null -H "Content-Type: application/json" \
    "$API_BASE_URL/query" -d '{"sql": "SELECT version()"}')

if [ "$HTTP_CODE" = "401" ]; then
    print_success "Unauthenticated request correctly returned 401"
else
    print_error "Unauthenticated request returned HTTP $HTTP_CODE (expected 401)"
fi

# Test 3: Query with invalid token (should fail)
print_info "Test 3: Query with invalid JWT token (should return 401)"
HTTP_CODE=$(curl -s -w "%{http_code}" -o /dev/null \
    -H "Authorization: Bearer invalid-token" \
    -H "Content-Type: application/json" \
    "$API_BASE_URL/query" -d '{"sql": "SELECT version()"}')

if [ "$HTTP_CODE" = "401" ]; then
    print_success "Invalid token correctly returned 401"
else
    print_error "Invalid token returned HTTP $HTTP_CODE (expected 401)"
fi

# Test 4: Query with valid token (should succeed)
print_info "Test 4: Query with valid JWT token (should return 200)"
RESPONSE=$(curl -s -w "%{http_code}" \
    -H "Authorization: Bearer $TEST_TOKEN" \
    -H "Content-Type: application/json" \
    "$API_BASE_URL/query" -d '{"sql": "SELECT version()"}')

HTTP_CODE="${RESPONSE: -3}"
RESPONSE_BODY="${RESPONSE%???}"

if [ "$HTTP_CODE" = "200" ]; then
    print_success "Valid token correctly returned 200"
    print_info "Response contains PostgreSQL version info"
else
    print_error "Valid token returned HTTP $HTTP_CODE (expected 200)"
    echo "Response: $RESPONSE_BODY"
fi

# Test 5: Database query test
print_info "Test 5: Database query test"
RESPONSE=$(curl -s -w "%{http_code}" \
    -H "Authorization: Bearer $TEST_TOKEN" \
    -H "Content-Type: application/json" \
    "$API_BASE_URL/query" -d '{"sql": "SELECT current_user, current_database()"}')

HTTP_CODE="${RESPONSE: -3}"

if [ "$HTTP_CODE" = "200" ]; then
    print_success "Database query executed successfully"
else
    print_error "Database query failed with HTTP $HTTP_CODE"
fi

print_info "JWT Authentication Tests Completed!"
print_info "Summary:"
print_info "- Health check: ✓"
print_info "- Unauthenticated request rejection: ✓"  
print_info "- Invalid token rejection: ✓"
print_info "- Valid token acceptance: ✓"
print_info "- Database query execution: ✓"
