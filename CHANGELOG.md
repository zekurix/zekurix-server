# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/2.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[unreleased]: https://github.com/zekurix/zekurix-server/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/zekurix/zekurix-server/releases/tag/v0.1.0