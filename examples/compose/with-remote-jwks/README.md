# Docker Compose deployment with remote JWKS

This example demonstrates a Docker Compose deployment of Zekurix Server using an external OAuth2/OIDC identity provider.

In this scenario:

- Zekurix Server and PostgreSQL run in Docker Compose.
- JWTs are issued by an external identity provider.
- Zekurix retrieves the JSON Web Key Set (JWKS) from the identity provider over HTTPS.
- Signing keys do not need to be bundled or mounted into the Zekurix container.

## Purpose

This example illustrates the recommended pattern when Zekurix is integrated with an existing identity provider.

Adapt the issuer, audience, and JWKS endpoint to your identity provider.

## Configuration

Authentication is configured in `zekurix.toml`:

```toml
[authentication]
issuer = [
    "https://auth.example.com/my-issuer"
]
audience = "zekurix"

[authentication.provider]
jwks_url = "https://auth.example.com/my-issuer/jwks"
```

The configured issuer must match the iss claim of accepted JWTs, and the audience must match the audience used by your identity provider for Zekurix.

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
