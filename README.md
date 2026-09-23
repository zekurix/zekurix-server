# Zekurix Server

[![All Contributors](https://img.shields.io/github/all-contributors/zekurix/zekurix-server?color=ee8449&style=flat-square)](#contributors)
[![OpenSSF Scorecard](https://api.securityscorecards.dev/projects/github.com/zekurix/zekurix-server/badge)](https://scorecard.dev/viewer/?uri=github.com/zekurix/zekurix-server)

**Zero-Knowledge Hierarchical Collaboration Platform - Server Component**

> [!WARNING]
> This project is in an early prototype stage.
> Core functionality is still under development and many planned features are not yet implemented.

## Project Scope

Zekurix Server is one component of the broader Zekurix ecosystem.

The project is intended to evolve into a larger ecosystem that may include:
- Zekurix Server
- Cross-platform SDK
- Web client
- Mobile clients
- Desktop clients

This repository contains only the backend server component.

## Planned Features

- Zero-knowledge encryption architecture
- Hierarchical permission model
- Group-based collaboration
- Vector clock-based synchronization
- Deterministic deletion through key shredding

## Getting Started

Clone the repository:

```bash
git clone https://github.com/zekurix/zekurix-server.git
cd zekurix-server
```

Set your environment variables:

```bash
cp .env.example .env
# Edit the .env file with your credentials
```

Setup the dependencies database:

```bash
docker compose up --build -d --wait
cargo install sqlx-cli --no-default-features --features postgres
sqlx migrate run
```

Build, test and run:

```bash
cargo build
cargo test
cargo run
```

If you modify the OpenAPI specification, you need to regenerate it:

```bash
./openapi/scripts/build_openapi.py
```

## Quick Start

### Running the full stack

To run all services (Zekurix server and its dependencies) with Docker Compose:

```bash
docker compose --profile full up --build -d --wait
```

### Shutting down

To stop and remove all containers:

```bash
docker compose --profile full down
```

Add the `-v` flag to also remove persistent data (volumes):

```bash
docker compose --profile full down -v
```

## Security Notice

> [!WARNING]
> Zekurix Server has not yet undergone an independent security audit. All security-related functionality should currently be considered experimental.

## Contributing

- See `CONTRIBUTING.md` for contribution guidelines and development setup instructions
- See `LICENSE.md` for full terms of the license
- Issues: https://github.com/zekurix/zekurix-server/issues

## Contributors

See `CONTRIBUTORS.md` for the complete list of project contributors.
