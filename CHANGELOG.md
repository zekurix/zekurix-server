# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/2.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2026-11-03

### Added

- JWT bearer authentication for API endpoints:
  - Validate issuer, audience, expiration, not-before, signature, and signing algorithm
  - Support multiple trusted JWT issuers
  - Support configurable asymmetric signing algorithms
  - Return standardized `401 Unauthorized` responses with `WWW-Authenticate: Bearer`
- Just-in-time user provisioning from authenticated identities
- `GET /api/v1/users/me` endpoint to retrieve the authenticated user
- External identity support using issuer and subject pairs
- JWKS-based JWT verification:
  - Load verification keys from local files or remote JWKS endpoints
  - Refresh remote keys automatically and support key rotation
  - Select verification keys by key ID (`kid`) and signing algorithm
  - Discard stale verification keys when refresh failures exceed the configured TTL
- Dependency-aware health checks reporting `ok`, `degraded`, and `unhealthy` states, with HTTP 503 returned when unhealthy
- `--dry-run` command to validate configuration without starting the server
- Docker Compose examples for remote JWKS and mock OAuth2 authentication

### Changed

- **Breaking:** User endpoints now require authentication
- **Breaking:** Users are now identified by external issuer and subject pairs instead of usernames
- OpenAPI specification is configured at runtime to expose the allowed authentication issuers

### Fixed

- Docker health checks now use the configured server port instead of a hardcoded port

## [0.2.0] - 2026-09-29

### Added

- Docker deployment support:
  - Official Docker image distribution through Docker Hub and GitHub Container Registry (GHCR)
  - Docker Compose deployment examples for common scenarios:
    - Minimal deployment
    - Environment variable based configuration
    - Combined environment variable and TOML configuration
  - Example configurations and deployment documentation
  - Published container images include SBOM and provenance attestations (SLSA) for improved software supply-chain transparency.
- Configuration:
  - Support for `ZEKURIX_CONFIG_PATH` environment variable to override the configuration file location

### Changed

- Default server bind address changed from `127.0.0.1` to `0.0.0.0` to improve container compatibility

## [0.1.0] - 2026-09-14

### Added

- User Management: Create and retrieve users with validation
  - `POST /api/v1/users` - Create a new user
  - `GET /api/v1/users/{id}` - Retrieve user details
  - Username format validation during JSON deserialization
  - HTTP 201 response with `Location` header on successful user creation
- Health Check Endpoint: `GET /health` returning JSON response with server status
- API Documentation: OpenAPI `3.1.0` specification with Redocly bundling
  - `GET /api/openapi.json` - Access the OpenAPI specification
  - Contract-first specification organized by domain
  - Full documentation of all user endpoints
- Error Handling: RFC 9457 Problem Details format for all error responses
- Request Tracing: X-Request-ID header support for distributed tracing
- Structured Logging: Request logging with tracing layer for observability
- Configuration File Support: TOML-based configuration with validation
  - Example configuration file (`zekurix.example.toml`)
  - Mandatory database credentials (username and password)
  - Unknown configuration fields are rejected
- Database Support: PostgreSQL persistence for users
  - Automatic database schema migration
  - Connection pooling and management
- Graceful Shutdown: Server gracefully closes active connections on shutdown
- Request Timeout: Configurable request timeout to prevent hanging requests

[unreleased]: https://github.com/zekurix/zekurix-server/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/zekurix/zekurix-server/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/zekurix/zekurix-server/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/zekurix/zekurix-server/releases/tag/v0.1.0
