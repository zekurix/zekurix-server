# Minimal Docker Compose deployment

This example demonstrates a minimal Docker Compose deployment for Zekurix Server.

In this scenario:

- The application runs from a published container image.
- All configuration is provided through environment variables.
- No `.env` file is used.
- No external configuration file is used.

## Purpose

- Demonstrate the minimum configuration required to run Zekurix Server.
- Provide a starting point for more advanced deployments.

## Usage

Set the required environment variables:
 
```bash
export ZEKURIX_DATABASE__PASSWORD=change-me
```

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
