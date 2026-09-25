# Docker Compose deployment with .env and configuration file

This example demonstrates a Docker Compose deployment for Zekurix Server using both a `.env` file and a TOML configuration file.

In this scenario:

- The application runs from a published container image.
- Infrastructure and deployment settings are provided through environment variables.
- Application settings are stored in a TOML configuration file.

## Purpose

- Demonstrate how to separate environment-specific values from application configuration.
- Store secrets and deployment settings in a `.env` file.
- Store application settings in a TOML configuration file.
- Provide a production-oriented deployment example.

## Usage

Review and adjust the environment variables in `.env` as required.

Review and adjust the application settings in `zekurix.toml` as required.

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
