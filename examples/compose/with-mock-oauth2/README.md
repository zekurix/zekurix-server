# Docker Compose deployment with mock OAuth2 server

This example demonstrates a self-contained Docker Compose deployment of Zekurix Server with a mock OAuth2/OIDC identity provider.

In this scenario:

- Zekurix Server, PostgreSQL, and the mock identity provider run in Docker Compose.
- The mock identity provider issues signed JWTs for development and testing.
- Zekurix retrieves JWKS directly from the mock identity provider.
- JWT subjects can be generated dynamically for different mock users.

## Purpose

This example provides a self-contained environment for developing and testing applications that integrate with Zekurix without requiring access to a real identity provider.

It is intended for local development and integration testing only.

## Usage

Review `.env` and `zekurix.toml` and adapt them to your environment.

Start the stack:

```bash
docker compose up -d --wait
```

Verify that the application is healthy:

```bash
curl --fail http://127.0.0.1:3000/health
```

The command should return an HTTP 200 response, indicating that the application started successfully.

Stop the stack:

```bash
docker compose down
```

## Request a token

Request a token for a mock user:

```bash
curl --silent \
    http://localhost:3001/my-issuer/token \
    --data 'grant_type=client_credentials' \
    --data 'client_id=test-client' \
    --data 'client_secret=test-secret' \
    --data 'username=Alice'
```

The issued access token contains `Alice` as its `sub` claim and can be used to send authenticated requests to Zekurix.
