# Contributing

Thank you for your interest in contributing to Zekurix Server.

## Project Status

> [!WARNING]
> Zekurix Server is currently in an early prototype stage.
> APIs, data models, and architectural decisions may change significantly before the first stable release.

## Guidelines

- Keep changes focused and easy to review.
- Follow the existing code style and conventions.
- Add or update tests when applicable.
- Update documentation when relevant.

## Pull Requests

Before opening a pull request, please ensure that the following checks pass successfully:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

All CI checks must pass before a pull request can be merged.

## Security

Please do not report security vulnerabilities through public GitHub issues.
See `SECURITY.md` for the vulnerability disclosure process.
