<!-- Use this file to provide workspace-specific custom instructions to Copilot. For more details, visit https://code.visualstudio.com/docs/copilot/copilot-customization#_use-a-githubcopilotinstructionsmd-file -->

# PostgreSQL OIDC Proxy - Copilot Instructions

This is a Rust project that implements a PostgreSQL proxy server with OIDC (OpenID Connect) authentication.

## Project Structure
- `src/main.rs` - Main application entry point and API routes
- `src/config.rs` - Configuration management
- `src/oidc.rs` - OIDC authentication and JWT validation
- `src/postgres.rs` - PostgreSQL connection pool management

## Key Dependencies
- `tokio` - Async runtime
- `axum` - Web framework
- `tokio-postgres` - PostgreSQL async client
- `jsonwebtoken` - JWT validation
- `serde` - Serialization/deserialization
- `reqwest` - HTTP client for OIDC endpoints

## Architecture Notes
- The proxy accepts HTTP POST requests with SQL queries
- All requests (except `/health`) require valid OIDC JWT tokens
- JWT tokens are validated against the configured OIDC provider's JWKS endpoint
- Database connections are managed through a connection pool with semaphore-based limiting
- Configuration can be provided via YAML file or environment variables

## Security Considerations
- Always validate JWT tokens before executing queries
- Use connection pooling to prevent resource exhaustion
- Log authentication attempts and query executions
- Consider implementing query whitelisting for production use

## Development Guidelines
- Follow Rust async/await patterns
- Use proper error handling with `anyhow` and `Result` types
- Implement comprehensive logging with `tracing`
- Ensure thread safety for shared state
