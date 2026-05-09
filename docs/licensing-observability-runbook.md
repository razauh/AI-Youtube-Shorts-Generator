# Licensing Observability Runbook

> **Document Classification: PROJECT-SPECIFIC**
> This file documents alerting for the Cloudflare Worker licensing backend deployment.
> For language-agnostic logging guidance, see: `enhanced-logging-module-guide.md`, `enhanced-logging-event-catalog.md`.

This runbook documents alerting for licensing backend logs emitted by the Cloudflare Worker.

## 0) Ownership and Escalation

- Primary owner: Licensing on-call engineer.
- Secondary owner: Platform on-call engineer.
- Escalation channel: `#licensing-incidents`.
- Escalation policy:
  - Critical: page immediately, acknowledge within 10 minutes.
  - High: page immediately, acknowledge within 15 minutes.
  - Medium: ticket + on-call notification, acknowledge within 60 minutes.

## 1) Log Event Format

Counter-style metrics are emitted as structured logs:

- `event = "metric.counter"`
- `metric = "<metric-name>"`
- `value = 1`
- Optional labels (for example `reason_code`, `route`, `scope`, `status`)

## 2) Metrics and Intent

- `licensing.provider.gumroad.misconfigured`
  - Worker is missing required Gumroad config.
- `licensing.provider.gumroad.http_non_200`
  - Gumroad returned non-2xx.
- `licensing.provider.gumroad.exception`
  - Fetch/timeout/runtime exception during Gumroad verification.
- `licensing.activate.denied`
  - Activation denied (`reason_code` label present).
- `licensing.validate.denied`
  - Validation denied (`reason_code` label present).
- `licensing.validate.revoked_hit`
  - Explicit revoked license encountered during validation.
- `licensing.validate.device_conflict_hit`
  - Device conflict encountered during validation.
- `licensing.rate_limit.hit`
  - Rate limit enforced (`route` and `scope` labels present).

## 3) Suggested Alerts

Use rolling windows and tune thresholds to your baseline traffic.

1. Provider outage signal:
   - Condition: `sum(metric=licensing.provider.gumroad.exception OR metric=licensing.provider.gumroad.http_non_200) >= 10 over 5m`
   - Severity: high
2. Misconfiguration detection:
   - Condition: `sum(metric=licensing.provider.gumroad.misconfigured) >= 1 over 5m`
   - Severity: critical
3. Revoke/device abuse tracking:
   - Condition: `sum(metric=licensing.validate.revoked_hit) >= 5 over 10m`
   - Severity: medium
   - Condition: `sum(metric=licensing.validate.device_conflict_hit) >= 10 over 10m`
   - Severity: medium
4. Denial spike:
   - Condition: `sum(metric=licensing.validate.denied) >= 30 over 10m`
   - Severity: medium
   - Break down by `reason_code`.
5. Rate-limit pressure:
   - Condition: `sum(metric=licensing.rate_limit.hit) >= 50 over 10m`
   - Severity: medium
   - Break down by `route` and `scope`.

## 3.1) Severity to Action Deadline

- `critical`: mitigation in progress within 15 minutes.
- `high`: mitigation in progress within 30 minutes.
- `medium`: mitigation plan and owner assigned within 4 hours.

## 3.2) SLO and Error Budget

- Availability SLO (license validate path): `99.9%` successful non-5xx responses per 30-day window.
- Latency SLO (activate path): `p95 < 400ms` excluding external provider latency.
- Error budget policy:
  - Budget burn > 50% in current 30-day window: freeze non-critical deploys to licensing worker.
  - Budget burn > 80%: incident review and rollback-ready posture required for all changes.

## 3.3) Baseline Guidance (starting points; tune to traffic)

- `licensing.validate.denied`: normal if stable and mostly `INVALID_KEY`; investigate if >2x 7-day moving average.
- `licensing.rate_limit.hit`: investigate if >3x 7-day moving average or concentrated on a single route.
- provider exceptions/non-200: investigate any sustained 5+ minutes elevation above 7-day moving average.

## 4) Dashboard Panels

1. Total provider failures by metric.
2. Validation denials split by `reason_code`.
3. Rate-limit hits split by `route` and `scope`.
4. Revoked and device-conflict counts over time.

## 5) Triage Quick Steps

1. If provider failure alerts fire:
   - Check Gumroad status and worker egress/network health.
   - Check Worker env vars (`GUMROAD_PRODUCT_ID`, auth token).
2. If revoke/device-conflict spikes fire:
   - Sample activation events for affected license IDs.
   - Confirm support reset patterns vs abuse patterns.
3. If denial spike fires with `INVALID_KEY`:
   - Verify client deployment and token handling path.
4. If rate-limit hits spike:
   - Inspect `route`/`scope` labels and consider limit tuning.
