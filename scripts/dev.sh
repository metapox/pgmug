#!/bin/bash

# Development and testing scripts for PostgreSQL OIDC Proxy

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check dependencies
check_dependencies() {
    print_status "Checking dependencies..."
    
    if ! command_exists cargo; then
        print_error "Rust/Cargo is not installed. Please install Rust first."
        exit 1
    fi
    
    if ! command_exists docker; then
        print_warning "Docker is not installed. Some features will not be available."
    fi
    
    if ! command_exists curl; then
        print_warning "curl is not installed. Health checks will not work."
    fi
}

# Build the project
build() {
    print_status "Building the project..."
    cargo build --release
}

# Run tests
test() {
    print_status "Running tests..."
    cargo test
}

# Start development server
dev() {
    print_status "Starting development server..."
    export RUST_LOG=debug
    cargo run
}

# Start with Docker Compose
docker_up() {
    print_status "Starting services with Docker Compose..."
    docker-compose up -d
    
    print_status "Waiting for services to be ready..."
    sleep 10
    
    # Check health
    if command_exists curl; then
        if curl -f http://localhost:8080/health >/dev/null 2>&1; then
            print_status "PostgreSQL OIDC Proxy is healthy!"
        else
            print_warning "Proxy health check failed. Check logs with: docker-compose logs postgres-proxy"
        fi
    fi
}

# Stop Docker services
docker_down() {
    print_status "Stopping Docker services..."
    docker-compose down
}

# View logs
logs() {
    docker-compose logs -f postgres-proxy
}

# Run health check
health_check() {
    if command_exists curl; then
        print_status "Checking proxy health..."
        if curl -f http://localhost:8080/health; then
            print_status "Health check passed!"
        else
            print_error "Health check failed!"
            exit 1
        fi
    else
        print_error "curl is required for health checks"
        exit 1
    fi
}

# Test API with sample requests (requires valid JWT)
test_api() {
    if [ -z "$JWT_TOKEN" ]; then
        print_error "JWT_TOKEN environment variable is required for API testing"
        print_warning "Example: export JWT_TOKEN=your_jwt_token"
        exit 1
    fi
    
    print_status "Testing API endpoints..."
    
    # Test health endpoint
    print_status "Testing health endpoint..."
    curl -f http://localhost:8080/health
    
    # Test query endpoint
    print_status "Testing query endpoint..."
    curl -X POST http://localhost:8080/query \
        -H "Authorization: Bearer $JWT_TOKEN" \
        -H "Content-Type: application/json" \
        -d '{"query": "SELECT id, name, email FROM users LIMIT 3"}'
    
    # Test mutation endpoint
    print_status "Testing mutation endpoint..."
    curl -X POST http://localhost:8080/mutation \
        -H "Authorization: Bearer $JWT_TOKEN" \
        -H "Content-Type: application/json" \
        -d '{"query": "INSERT INTO users (name, email) VALUES ($1, $2)", "params": ["Test User", "test@example.com"]}'
}

# Clean up build artifacts
clean() {
    print_status "Cleaning build artifacts..."
    cargo clean
    docker-compose down -v 2>/dev/null || true
}

# Show usage
usage() {
    echo "PostgreSQL OIDC Proxy Development Scripts"
    echo ""
    echo "Usage: $0 <command>"
    echo ""
    echo "Commands:"
    echo "  check-deps    Check if all dependencies are installed"
    echo "  build         Build the project"
    echo "  test          Run tests"
    echo "  dev           Start development server"
    echo "  docker-up     Start services with Docker Compose"
    echo "  docker-down   Stop Docker services"
    echo "  logs          View proxy logs"
    echo "  health        Run health check"
    echo "  test-api      Test API endpoints (requires JWT_TOKEN env var)"
    echo "  clean         Clean build artifacts and Docker volumes"
    echo "  help          Show this help message"
}

# Main script logic
case "${1:-}" in
    check-deps)
        check_dependencies
        ;;
    build)
        check_dependencies
        build
        ;;
    test)
        check_dependencies
        test
        ;;
    dev)
        check_dependencies
        dev
        ;;
    docker-up)
        docker_up
        ;;
    docker-down)
        docker_down
        ;;
    logs)
        logs
        ;;
    health)
        health_check
        ;;
    test-api)
        test_api
        ;;
    clean)
        clean
        ;;
    help|--help|-h)
        usage
        ;;
    *)
        print_error "Unknown command: ${1:-}"
        echo ""
        usage
        exit 1
        ;;
esac
