# Zekurix Server

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

Setup the PostgreSQL database:

```bash
docker run --name zekurix-postgres -e POSTGRES_USER=postgres -e POSTGRES_PASSWORD=<password> -e POSTGRES_DB=zekurix -p 5432:5432 -d postgres:18
cargo install sqlx-cli --no-default-features --features postgres
sqlx migrate run
```

Build, test and run:

```bash
cargo build
cargo test
cargo run
```

## Security Notice

> [!WARNING]
> Zekurix Server has not yet undergone an independent security audit. All security-related functionality should currently be considered experimental.

## Contributing

- See `CONTRIBUTING.md` for contribution guidelines and development setup instructions
- See `LICENSE.md` for full terms of the license
- Issues: https://github.com/zekurix/zekurix-server/issues
