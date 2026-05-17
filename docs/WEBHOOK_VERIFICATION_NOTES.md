# Webhook Verification Notes

## Status

As of May 18, 2026:

- Worker deploy succeeded.
- Health check succeeded:
  - `GET /health` -> `{"ok":true,"data":{"status":"ok","contract":"v1"}}`

## Deferred Live Gumroad Verification

Live Gumroad webhook verification is deferred until real sale data is available.

Test run with placeholder values:

```bash
curl -i -X POST https://license-worker.demandscout.workers.dev/v1/license/webhooks/gumroad \
  -H "content-type: application/x-www-form-urlencoded" \
  --data "sale_id=YOUR_SALE_ID&product_id=YOUR_PRODUCT_ID&email=BUYER_EMAIL"
```

Observed response:

- HTTP `503`
- `error.code = "serialization"`
- message: `Gumroad verification payload is missing required sale fields.`

This is expected when using non-real placeholder sale values.

## Later Validation Step

Rerun webhook test with real Gumroad data:

- real `sale_id`
- matching `product_id`
- matching purchaser `email`

Command:

```bash
curl -i -X POST https://license-worker.demandscout.workers.dev/v1/license/webhooks/gumroad \
  -H "content-type: application/x-www-form-urlencoded" \
  --data "sale_id=<REAL_SALE_ID>&product_id=<REAL_PRODUCT_ID>&email=<REAL_BUYER_EMAIL>"
```
