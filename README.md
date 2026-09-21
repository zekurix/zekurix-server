# Zekurix Server

[![All Contributors](https://img.shields.io/github/all-contributors/projectOwner/projectName?color=ee8449&style=flat-square)](#contributors)
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

If you modify the OpenAPI specification, you need to regenerate it:
```bash
./openapi/scripts/build_openapi.py
```

If you want to run the service fully with docker compose:
```bash
docker compose up --build
# ...
docker compose down -v
```

## Security Notice

> [!WARNING]
> Zekurix Server has not yet undergone an independent security audit. All security-related functionality should currently be considered experimental.

## Contributing

- See `CONTRIBUTING.md` for contribution guidelines and development setup instructions
- See `LICENSE.md` for full terms of the license
- Issues: https://github.com/zekurix/zekurix-server/issues

## Contributors

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tbody>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/simon-batardiere"><img src="https://avatars.githubusercontent.com/u/47428512?v=4?s=100" width="100px;" alt="Simon Batardiere"/><br /><sub><b>Simon Batardiere</b></sub></a><br /><a href="#bug-simon-batardiere" title="Bug reports">🐛</a> <a href="#code-simon-batardiere" title="Code">💻</a> <a href="#doc-simon-batardiere" title="Documentation">📖</a> <a href="#example-simon-batardiere" title="Examples">💡</a> <a href="#ideas-simon-batardiere" title="Ideas, Planning, & Feedback">🤔</a> <a href="#infra-simon-batardiere" title="Infrastructure (Hosting, Build-Tools, etc)">🚇</a> <a href="#maintenance-simon-batardiere" title="Maintenance">🚧</a> <a href="#review-simon-batardiere" title="Reviewed Pull Requests">👀</a> <a href="#security-simon-batardiere" title="Security">🛡️</a> <a href="#test-simon-batardiere" title="Tests">⚠️</a> <a href="#tutorial-simon-batardiere" title="Tutorials">✅</a></td>
    </tr>
  </tbody>
</table>

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->

<!-- ALL-CONTRIBUTORS-LIST:END -->
