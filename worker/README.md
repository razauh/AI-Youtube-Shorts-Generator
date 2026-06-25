# License Worker (Cloudflare)

This worker implements companion licensing operations exercised by:

- `worker/test/contract.test.js`
- `tests/fixtures/license_worker_contract_v1/`

Production release configuration is centralized in [`docs/release-production-config.md`](../docs/release-production-config.md).

## Routes

- `GET /health`
- `GET /readyz`
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
wrangler secret put HASH_PEPPER
```

## Notes

- Production desktop activation and validation use Devolens directly.
- Worker routes are D1-backed for:
  - user data deletion request/status/admin review
  - Gumroad webhook idempotency/audit/license upsert
  - admin reset review and device-binding actions
- Gumroad webhook verification now uses server-to-server sale verification with `GUMROAD_ACCESS_TOKEN`.
- Gumroad webhook verification expects Gumroad `sale.id` as `sale_id`. Do not use Gumroad `order_id`.
- License key hashing uses `HASH_PEPPER`.
- Customer updater checks use `UPDATE_MANIFEST_URL` and return Tauri updater JSON directly, not the license API envelope.
