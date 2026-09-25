# Docker Compose deployment with .env

This example demonstrates a Docker Compose deployment for Zekurix Server using a `.env` file.

In this scenario:

- The application runs from a published container image.
- Configuration is provided through environment variables.
- Environment variables are stored in a `.env` file.
- No external configuration file is used.

## Purpose

- Demonstrate how to manage environment variables using a `.env` file.
- Avoid passing environment variables directly from the command line.
- Provide a starting point for more advanced deployments.

## Usage

Review and adjust the environment variables in `.env` as required.

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
