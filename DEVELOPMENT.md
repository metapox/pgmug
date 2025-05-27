# Development Guide

This guide provides detailed information for developers working on the PostgreSQL OIDC Proxy.

## Architecture Overview

The PostgreSQL OIDC Proxy consists of four main modules:

### 1. Main Application (`src/main.rs`)
- HTTP server setup using Axum framework
- Route definitions and request handling
- Application lifecycle management
- Integration of all components

### 2. Configuration Management (`src/config.rs`)
- YAML and environment variable configuration loading
- Configuration validation and defaults
- Hierarchical configuration support

### 3. OIDC Authentication (`src/oidc.rs`)
- JWT token validation
- JWKS (JSON Web Key Set) fetching and caching
- Authentication middleware
- Claims extraction and validation

### 4. PostgreSQL Connection (`src/postgres.rs`)
- Connection pool management
- Semaphore-based connection limiting
- Query execution with proper error handling
- Connection health monitoring

## Development Setup

### Prerequisites
- Rust 1.80.0 or later
- PostgreSQL database
- OIDC provider (Auth0, Keycloak, etc.)

### Quick Start
```bash
# Clone and setup
git clone <repository-url>
cd postgres-oidc-proxy

# Copy environment configuration
cp .env.development .env
# Edit .env with your settings

# Run development server
./scripts/dev.sh dev

# In another terminal, test health endpoint
curl http://localhost:8080/health
```

## Configuration Details

### Environment Variables
All configuration options can be set via environment variables with the `POSTGRES_PROXY_` prefix:

#### Server Configuration
- `POSTGRES_PROXY_SERVER__BIND_ADDRESS`: Server bind address (default: `0.0.0.0:8080`)

#### Database Configuration
- `POSTGRES_PROXY_DATABASE__HOST`: PostgreSQL host (default: `localhost`)
- `POSTGRES_PROXY_DATABASE__PORT`: PostgreSQL port (default: `5432`)
- `POSTGRES_PROXY_DATABASE__USERNAME`: Database username
- `POSTGRES_PROXY_DATABASE__PASSWORD`: Database password
- `POSTGRES_PROXY_DATABASE__DATABASE`: Database name
- `POSTGRES_PROXY_DATABASE__MAX_CONNECTIONS`: Max concurrent connections (default: `10`)

#### OIDC Configuration
- `POSTGRES_PROXY_OIDC__ISSUER_URL`: OIDC issuer URL
- `POSTGRES_PROXY_OIDC__CLIENT_ID`: OIDC client ID
- `POSTGRES_PROXY_OIDC__AUDIENCE`: Expected token audience (optional)
- `POSTGRES_PROXY_OIDC__JWKS_CACHE_DURATION_SECONDS`: JWKS cache duration (default: `3600`)

### YAML Configuration
Alternatively, use `config.yaml`:

```yaml
server:
  bind_address: "0.0.0.0:8080"

database:
  host: "localhost"
  port: 5432
  username: "postgres"
  password: "password"
  database: "postgres"
  max_connections: 10

oidc:
  issuer_url: "https://your-oidc-provider.com"
  client_id: "your-client-id"
  audience: "your-audience"
  jwks_cache_duration_seconds: 3600
```

## API Endpoints

### Health Check
```http
GET /health
```
Returns server health status. No authentication required.

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2025-05-27T12:00:00Z"
}
```

### Execute Query
```http
POST /query
Authorization: Bearer <JWT_TOKEN>
Content-Type: application/json

{
  "query": "SELECT * FROM users LIMIT 10"
}
```

Executes a read-only SQL query.

**Response:**
```json
{
  "columns": ["id", "name", "email"],
  "rows": [
    [1, "Alice", "alice@example.com"],
    [2, "Bob", "bob@example.com"]
  ]
}
```

### Execute Mutation
```http
POST /mutation
Authorization: Bearer <JWT_TOKEN>
Content-Type: application/json

{
  "query": "INSERT INTO users (name, email) VALUES ($1, $2)",
  "params": ["John Doe", "john@example.com"]
}
```

Executes a write SQL query (INSERT, UPDATE, DELETE).

**Response:**
```json
{
  "rows_affected": 1
}
```

## Testing

### Unit Tests
```bash
cargo test
```

### Integration Testing with Docker
```bash
# Start test environment
./scripts/dev.sh docker-up

# Run API tests (requires JWT_TOKEN)
export JWT_TOKEN="your-test-token"
./scripts/dev.sh test-api

# Clean up
./scripts/dev.sh docker-down
```

### Manual Testing
```bash
# Generate test JWT (development only)
./scripts/generate_test_jwt.sh

# Test health endpoint
curl http://localhost:8080/health

# Test query endpoint
curl -X POST http://localhost:8080/query \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"query": "SELECT version()"}'
```

## Security Considerations

### JWT Validation
- All tokens are validated against OIDC provider's JWKS
- Token expiration is checked
- Audience validation (if configured)
- Signature verification using public keys

### Database Security
- Connection pooling prevents resource exhaustion
- Parameterized queries prevent SQL injection
- Connection limits protect database resources

### Production Deployment
- Use HTTPS for all connections
- Implement rate limiting
- Use proper database user permissions
- Enable comprehensive logging
- Consider query whitelisting

## Development Scripts

The `scripts/dev.sh` script provides common development tasks:

```bash
./scripts/dev.sh check-deps    # Check dependencies
./scripts/dev.sh build         # Build project
./scripts/dev.sh test          # Run tests
./scripts/dev.sh dev           # Start development server
./scripts/dev.sh docker-up     # Start with Docker Compose
./scripts/dev.sh docker-down   # Stop Docker services
./scripts/dev.sh logs          # View logs
./scripts/dev.sh health        # Run health check
./scripts/dev.sh test-api      # Test API endpoints
./scripts/dev.sh clean         # Clean build artifacts
```

## Error Handling

The application uses structured error handling:

- **Authentication Errors**: Invalid or expired JWT tokens
- **Database Errors**: Connection failures, SQL errors
- **Configuration Errors**: Invalid or missing configuration
- **Network Errors**: OIDC provider connectivity issues

All errors are logged with appropriate detail levels and return user-friendly HTTP responses.

## Performance Optimization

### Connection Pooling
- Semaphore-based connection limiting
- Async connection management
- Connection health monitoring

### Caching
- JWKS endpoint caching with configurable TTL
- Connection reuse
- Minimal memory allocation

### Async I/O
- Tokio-based async runtime
- Non-blocking database operations
- Concurrent request handling

## Monitoring and Observability

### Logging
Uses structured logging with `tracing`:

```bash
# Set log level
export RUST_LOG=debug  # trace, debug, info, warn, error

# Run with detailed logging
cargo run
```

### Metrics
Consider adding metrics for:
- Request rate and latency
- Database connection pool usage
- Authentication success/failure rates
- Error rates by type

### Health Checks
The `/health` endpoint provides basic health status. Consider extending with:
- Database connectivity check
- OIDC provider reachability
- Resource usage metrics

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/new-feature`
3. Make your changes
4. Add tests for new functionality
5. Run tests: `cargo test`
6. Check formatting: `cargo fmt`
7. Run clippy: `cargo clippy`
8. Commit your changes: `git commit -am 'Add new feature'`
9. Push to the branch: `git push origin feature/new-feature`
10. Submit a pull request

## Troubleshooting

### Common Issues

1. **Server won't start**
   - Check configuration file format
   - Verify environment variables
   - Check port availability

2. **Database connection failed**
   - Verify PostgreSQL is running
   - Check connection parameters
   - Verify user permissions

3. **JWT validation failed**
   - Check OIDC issuer URL
   - Verify client ID configuration
   - Check token format and expiration

4. **JWKS fetch failed**
   - Verify OIDC provider URL
   - Check network connectivity
   - Review provider's JWKS endpoint

### Debug Mode
```bash
RUST_LOG=debug cargo run
```

This provides detailed logging for troubleshooting issues.
