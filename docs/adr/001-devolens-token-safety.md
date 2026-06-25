# ADR 001: Devolens Token Safety and Production Architecture Decision

## Context
The application is migrating its licensing backend from a custom Cloudflare Worker to Devolens (Cryptolens). 
Devolens access tokens do not encode their permission scopes in the token string itself. This makes it impossible to verify the scopes of a token offline.

A critical security requirement is that **Tauri Desktop Client Tokens (Client-Facing)** must only have `Activate` and `Deactivate` scopes, and **must not** have administrative scopes like `Create Key`, `Block Key`, `Get Keys`, or `Get Products`. Shipping a broad management/administrative token inside a desktop client would allow users to extract the token and generate or modify arbitrary licenses.

## Decision
We will enforce token scope safety at startup for the Devolens customer licensing path.

1. **Active Validation Check**:
   During configuration validation, the application will attempt to query the Devolens `/api/product/GetProducts` endpoint with the loaded token.
   - **If the query succeeds** (returns HTTP 200 with product information): The token has management scopes and is **too privileged**. The config loader will throw `ConfigError::InvalidConfigValue` and refuse to start the application.
   - **If the query fails with Access Denied** (meaning the token lacks the scope to list products): The token is verified to be safe/client-only. Config loading will proceed.
   - **If the network is unreachable** (DNS failure, request timeout): The check fails open or logs a warning but does not block startup, ensuring offline capability.

2. **Worker Boundary**:
   The Cloudflare Worker remains a companion service for Gumroad webhooks, updater checks, privacy/admin operations, D1-backed records, migrations, and admin desktop access. It is not a customer activation, validation, or public reset backend.

## Consequences
- Prevents accidental distribution or usage of privileged administrative tokens in the Tauri desktop app.
- Introduces a lightweight, one-time network check at configuration load time when Devolens credentials are configured.
- Fails safe by allowing offline launching when Devolens API is unreachable.
