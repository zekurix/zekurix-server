# Zekurix Server

**Zero-Knowledge Hierarchical Collaboration Platform - Server Component**

## Overview

Zekurix Server is the backend component of the Zekurix platform. It provides encrypted blob storage, permission validation, and synchronization coordination while maintaining zero-knowledge guarantees. The server never accesses plaintext user data.

## Key Features

- Zero-knowledge encryption architecture
- Permission validation at API boundary without decryption capability
- User and nested group hierarchy support for flexible permission management
- Vector clock-based synchronization with conflict detection
- Deterministic deletion via key shredding
- Per-user quota enforcement and session management

## Contributing

- See `CONTRIBUTING.md` for contribution guidelines and development setup instructions
- See `LICENSE.md` for full terms of the license
- Issues: https://github.com/zekurix/zekurix-server/issues
