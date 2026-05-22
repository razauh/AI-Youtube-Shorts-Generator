# Webhook Verification Notes

## Status

As of May 18, 2026:

- Worker deploy succeeded.
- Health check succeeded:
  - `GET /health` -> `{"ok":true,"data":{"status":"ok","contract":"v1"}}`
- Live Gumroad webhook verification succeeded with a real purchase:
  - `POST /v1/license/webhooks/gumroad` -> HTTP `200`
  - `{"ok":true,"data":{"accepted":true,"provider":"gumroad","sale_id":"<REAL_SALE_ID>","verified":true}}`

## Important Identifier Mapping

For this worker, the webhook payload must use the Gumroad sale API identifier:

- `sale_id` = `sale.id`
- `product_id` = `sale.product_id`
- `email` = purchaser email from the sale record

Do not use:

- `order_id`

The Gumroad dashboard may show an `Order ID`, but the worker verifies against:

```text
GET /v2/sales/<sale_id>
```

If `order_id` is used in place of `sale.id`, Gumroad can respond with:

```json
{"success":false,"message":"The sale was not found."}
```

## Initial Placeholder Test

Before real sale data was available, a placeholder test was run.

Command:

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

## Working Validation Flow

1. Confirm worker health:

```bash
curl -i https://license-worker.demandscout.workers.dev/health
```

Expected:

- HTTP `200`
- `{"ok":true,"data":{"status":"ok","contract":"v1"}}`

2. Look up the real sale in Gumroad:

```bash
curl -i "https://api.gumroad.com/v2/sales/<REAL_SALE_ID>?access_token=<GUMROAD_ACCESS_TOKEN>"
```

3. Use values from the Gumroad response:

- `sale_id` from `sale.id`
- `product_id` from `sale.product_id`
- `email` from the purchaser email field on the sale record

4. Send the webhook verification request:

```bash
curl -i -X POST https://license-worker.demandscout.workers.dev/v1/license/webhooks/gumroad \
  -H "content-type: application/x-www-form-urlencoded" \
  --data "sale_id=<REAL_SALE_ID>&product_id=<REAL_PRODUCT_ID>&email=<REAL_BUYER_EMAIL>"
```

Expected success response:

- HTTP `200`
- `{"ok":true,"data":{"accepted":true,"provider":"gumroad","sale_id":"<REAL_SALE_ID>","verified":true}}`

## Post-Deploy Verification

After deploying worker changes, verify both the success path and the improved failure path.

1. Confirm the success path still works with a real sale:

```bash
curl -i -X POST https://license-worker.demandscout.workers.dev/v1/license/webhooks/gumroad \
  -H "content-type: application/x-www-form-urlencoded" \
  --data "sale_id=<REAL_SALE_ID>&product_id=<REAL_PRODUCT_ID>&email=<REAL_BUYER_EMAIL>"
```

2. Confirm the failure path with an intentionally invalid `sale_id` and the same real `product_id` and `email`:

```bash
curl -i -X POST https://license-worker.demandscout.workers.dev/v1/license/webhooks/gumroad \
  -H "content-type: application/x-www-form-urlencoded" \
  --data "sale_id=definitely-wrong-sale-id&product_id=<REAL_PRODUCT_ID>&email=<REAL_BUYER_EMAIL>"
```

Expected on the second call:

- HTTP `404`
- `error.code = "not_found"`
- message: `Gumroad sale was not found for the provided sale_id.`
