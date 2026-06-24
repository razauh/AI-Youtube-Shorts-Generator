# ADR 001: Devolens Token Safety and Production Architecture Decision

## Context
The application is migrating its licensing backend from a custom Cloudflare Worker to Devolens (Cryptolens). 
Devolens access tokens do not encode their permission scopes in the token string itself. This makes it impossible to verify the scopes of a token offline.

A critical security requirement is that **Tauri Desktop Client Tokens (Client-Facing)** must only have `Activate` and `Deactivate` scopes, and **must not** have administrative scopes like `Create Key`, `Block Key`, `Get Keys`, or `Get Products`. Shipping a broad management/administrative token inside a desktop client would allow users to extract the token and generate or modify arbitrary licenses.

## Decision
We will enforce token scope safety at startup when the application is configured to use Devolens directly (`LICENSE_BACKEND_MODE=devolens`) in a Direct Tauri-to-Devolens communication style.

1. **Active Validation Check**:
   During configuration validation, the application will attempt to query the Devolens `/api/product/GetProducts` endpoint with the loaded token.
   - **If the query succeeds** (returns HTTP 200 with product information): The token has management scopes and is **too privileged**. The config loader will throw `ConfigError::InvalidConfigValue` and refuse to start the application.
   - **If the query fails with Access Denied** (meaning the token lacks the scope to list products): The token is verified to be safe/client-only. Config loading will proceed.
   - **If the network is unreachable** (DNS failure, request timeout): The check fails open or logs a warning but does not block startup, ensuring offline capability.

2. **Alternative Architecture (Thin Backend Proxy)**:
   If the developer wants to completely avoid distributing any Devolens tokens in the client, they can configure the app to route requests through a thin companion service/proxy (running on Cloudflare Workers) by setting `LICENSE_BACKEND_MODE=hosted` and setting `DEVOLENS_ACCESS_TOKEN` only on the server.

## Consequences
- Prevents accidental distribution or usage of privileged administrative tokens in the Tauri desktop app.
- Introduces a lightweight, one-time network check at configuration load time when direct Devolens mode is configured.
- Fails safe by allowing offline launching when Devolens API is unreachable.
