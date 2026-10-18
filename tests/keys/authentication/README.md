# Test Keys

Cryptographic keys used to authenticate users in local development and automated tests.

Tokens are signed with `private.pem` and verified by the server against the
public keys in `jwks.json`, mimicking the behaviour of a real identity provider.

These keys are NOT for production use.

## Contents

- `jwks.json` — public keys, loaded by the test setup
- `private.pem` — private key, used to sign test tokens

## Regeneration

Replace both files together: the JWKS must always match the private key.
