# License Worker (Cloudflare)

This worker implements the hosted licensing contract exercised by:

- `worker/test/contract.test.js`
- `tests/fixtures/license_worker_contract_v1/`

Production release configuration is centralized in [`docs/release-production-config.md`](../docs/release-production-config.md).

## Routes

- `GET /health`
- `GET /readyz`
- `POST /v1/license/activate`
- `POST /v1/license/validate`
- `POST /v1/license/reset/request`
- `POST /v1/license/reset/status`
- `POST /v1/privacy/delete/request`
- `POST /v1/privacy/delete/status`
- `GET /v1/admin/privacy/delete-requests`
- `POST /v1/admin/privacy/delete/approve`
- `POST /v1/admin/privacy/delete/reject`
- `POST /v1/license/webhooks/gumroad`
- `GET /updates/:target/:arch/:current_version`

## Run tests

From repo root:

```bash
pnpm run worker:test
```

Or directly:

```bash
node --test worker/test/contract.test.js
```

## Run locally with Wrangler

Install Wrangler in `worker/` and run:

```bash
pnpm install --frozen-lockfile
pnpm run worker:dev
```

Set required secret first:

```bash
wrangler secret put GUMROAD_ACCESS_TOKEN
wrangler secret put TOKEN_SIGNING_SECRET
wrangler secret put HASH_PEPPER
```

## Notes

- Production desktop licensing defaults to `https://license-worker.demandscout.workers.dev`.
- Worker routes are D1-backed for:
  - activate
  - validate
  - reset request
  - reset status
  - user data deletion request/status/admin review
  - Gumroad webhook idempotency/audit/license upsert
- Gumroad webhook verification now uses server-to-server sale verification with `GUMROAD_ACCESS_TOKEN`.
- Gumroad webhook verification expects Gumroad `sale.id` as `sale_id`. Do not use Gumroad `order_id`.
- Access tokens are signed/verified with `TOKEN_SIGNING_SECRET`.
- License key hashing uses `HASH_PEPPER` and is independent of token signing.
- Customer updater checks use `UPDATE_MANIFEST_URL` and return Tauri updater JSON directly, not the license API envelope.
