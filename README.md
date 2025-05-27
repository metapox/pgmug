# PostgreSQL OIDC Proxy

A high-performance PostgreSQL proxy server written in Rust that provides secure access to PostgreSQL databases through OIDC (OpenID Connect) authentication.

## Features

- **OIDC Authentication**: Secure JWT-based authentication using OpenID Connect
- **PostgreSQL Proxy**: Forward SQL queries to PostgreSQL databases
- **Connection Pooling**: Efficient connection management with configurable limits
- **RESTful API**: Simple HTTP API for executing queries and mutations
- **Configuration Flexibility**: Support for YAML files and environment variables
- **Comprehensive Logging**: Structured logging with tracing
- **Health Checks**: Built-in health check endpoint

## API Endpoints

### Health Check
```
GET /health
```
Returns server health status (no authentication required).

### Query Execution
```
POST /query
Authorization: Bearer <JWT_TOKEN>
Content-Type: application/json

{
  "query": "SELECT * FROM users WHERE id = 1"
}
```

### Mutation Execution
```
POST /mutation
Authorization: Bearer <JWT_TOKEN>
Content-Type: application/json

{
  "query": "INSERT INTO users (name, email) VALUES ($1, $2)",
  "params": ["John", "john@example.com"]
}
```

## Configuration

### YAML Configuration (config.yaml)
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

### Environment Variables
All configuration can be overridden using environment variables with the prefix `POSTGRES_PROXY_`:

```bash
POSTGRES_PROXY_SERVER__BIND_ADDRESS=0.0.0.0:8080
POSTGRES_PROXY_DATABASE__HOST=localhost
POSTGRES_PROXY_DATABASE__PORT=5432
POSTGRES_PROXY_DATABASE__USERNAME=postgres
POSTGRES_PROXY_DATABASE__PASSWORD=password
POSTGRES_PROXY_DATABASE__DATABASE=postgres
POSTGRES_PROXY_DATABASE__MAX_CONNECTIONS=10
POSTGRES_PROXY_OIDC__ISSUER_URL=https://your-oidc-provider.com
POSTGRES_PROXY_OIDC__CLIENT_ID=your-client-id
POSTGRES_PROXY_OIDC__AUDIENCE=your-audience
POSTGRES_PROXY_OIDC__JWKS_CACHE_DURATION_SECONDS=3600
```

## Getting Started

### Prerequisites
- Rust (latest stable version)
- PostgreSQL database
- OIDC provider (Auth0, Keycloak, etc.)

### Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd postgres-oidc-proxy
```

2. Build the project:
```bash
cargo build --release
```

3. Configure the application:
   - Copy `.env.example` to `.env` and update values
   - Or create `config.yaml` with your settings

4. Run the application:
```bash
cargo run
```

### Development

To run in development mode with auto-reload:
```bash
cargo watch -x run
```

To run tests:
```bash
cargo test
```

## Security Considerations

- **JWT Validation**: All tokens are validated against the OIDC provider's JWKS endpoint
- **Connection Limits**: Database connections are limited to prevent resource exhaustion
- **SQL Injection**: Consider implementing query whitelisting for production use
- **HTTPS**: Always use HTTPS in production environments
- **Token Rotation**: JWKS keys are cached and automatically refreshed

## Architecture

The proxy consists of several key components:

- **Main Server** (`main.rs`): HTTP server and routing
- **OIDC Module** (`oidc.rs`): JWT validation and OIDC integration
- **PostgreSQL Module** (`postgres.rs`): Database connection pooling
- **Configuration** (`config.rs`): Settings management

## Performance

- Async/await throughout for high concurrency
- Connection pooling to reduce database overhead
- JWKS caching to minimize external requests
- Structured logging for observability

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

For issues and questions, please use the GitHub issue tracker.
