# Shared Network Client Abstraction (Language-Agnostic Guide)

This document specifies the shared network client used by all desktop application modules for HTTP communication. It ensures consistent timeout, retry, offline detection, and error handling behavior.

---

## Purpose

All modules that make HTTP requests (Updater, Licensing, Admin Console) must use a shared network client abstraction rather than making raw HTTP calls. This ensures:
- Consistent timeout and retry behavior
- Unified offline detection
- Centralized error mapping to the shared error taxonomy
- Consistent logging of network events

---

## Interface

```
NetworkClient {
  request(config: RequestConfig) → Result<Response, NetworkError>
  isOnline() → boolean
  onConnectivityChange(callback: (online: boolean) → void) → Unsubscribe
}
```

### RequestConfig

| Field | Type | Required | Description |
|---|---|---|---|
| `method` | String | Yes | HTTP method (`GET`, `POST`, `PUT`, `PATCH`, `DELETE`) |
| `url` | String | Yes | Full URL |
| `headers` | Map<String, String> | No | Request headers |
| `body` | String / Bytes | No | Request body (JSON string or binary) |
| `timeout_ms` | Number | No | Per-request timeout override (default: from config) |
| `retry_policy` | RetryPolicy | No | Per-request retry override (default: from config) |
| `on_progress` | Callback | No | Progress callback for downloads |

### Response

| Field | Type | Description |
|---|---|---|
| `status` | Number | HTTP status code |
| `headers` | Map<String, String> | Response headers |
| `body` | String / Bytes | Response body |

---

## Timeout Configuration

| Parameter | Default | Description |
|---|---|---|
| `connect_timeout_ms` | 10,000 (10s) | Maximum time to establish connection |
| `read_timeout_ms` | 30,000 (30s) | Maximum time to receive response |
| `total_timeout_ms` | 60,000 (60s) | Maximum total time for the request |

- Timeouts must be configurable per-environment.
- Download operations (updater) may use longer timeouts (recommended: `300,000 ms` / 5 minutes).

---

## Retry Policy

### Default Policy

| Parameter | Default | Description |
|---|---|---|
| `max_attempts` | 3 | Total attempts (including initial) |
| `backoff_base_ms` | 1,000 | Base delay for exponential backoff |
| `backoff_max_ms` | 30,000 | Maximum delay cap |
| `jitter` | true | Add randomized jitter to prevent thundering herd |
| `retryable_status_codes` | `[429, 502, 503, 504]` | HTTP status codes that trigger retry |
| `retryable_errors` | `[NET_TIMEOUT, NET_CONNECTION_REFUSED]` | Error codes that trigger retry |

### Backoff Calculation

```
delay = min(backoff_base_ms * 2^(attempt - 1), backoff_max_ms)
if jitter:
  delay = delay * random(0.5, 1.5)
```

### Non-Retryable Conditions
- HTTP `400`, `401`, `403`, `404`, `405` — client errors are never retried.
- `SEC_*` errors — security failures are never retried.
- `VAL_*` errors — validation failures are never retried.
- HTTP `429` — retried only after `Retry-After` duration (not with backoff).

---

## Offline Detection

### Detection Methods
1. **Proactive**: Monitor OS-level network status change events where available.
2. **Reactive**: After a network error (`NET_TIMEOUT`, `NET_DNS_FAILURE`, `NET_CONNECTION_REFUSED`), mark as offline.
3. **Recovery**: Periodically attempt a lightweight connectivity check (e.g., HEAD request to a known endpoint) every 30 seconds when offline.

### Offline Behavior
- When offline, network requests that are not critical can be queued and retried when connectivity is restored.
- Critical requests (e.g., license validation) should fail immediately with `NET_OFFLINE` rather than waiting for timeout.
- Emit `network.offline.detected` log event when transitioning to offline.
- Emit `network.online.restored` log event when transitioning back to online.

---

## Error Mapping

Map all network-level errors to the shared error taxonomy:

| Network Condition | Error Code | Log Level |
|---|---|---|
| DNS resolution failure | `NET_DNS_FAILURE` | ERROR |
| Connection refused | `NET_CONNECTION_REFUSED` | ERROR |
| Connection timeout | `NET_TIMEOUT` | WARN |
| Read timeout | `NET_TIMEOUT` | WARN |
| TLS/SSL error | `NET_SSL_ERROR` | ERROR |
| No connectivity | `NET_OFFLINE` | WARN |
| HTTP 4xx | Map to `VAL_*`, `AUTH_*`, or `AUTHZ_*` based on status | WARN |
| HTTP 429 | `RATE_IP_EXCEEDED` or `RATE_KEY_EXCEEDED` | WARN |
| HTTP 5xx | `SYS_UNAVAILABLE` or `PROV_ERROR` | ERROR |

---

## Logging

All network operations must emit structured log events:

| Event | Level | Required Metadata |
|---|---|---|
| `network.request.timeout` | WARN | `url_host`, `duration_ms` |
| `network.request.dns_failure` | ERROR | `url_host` |
| `network.request.connection_refused` | ERROR | `url_host`, `port` |
| `network.request.ssl_error` | ERROR | `url_host`, `error_message` |
| `network.offline.detected` | WARN | — |
| `network.online.restored` | INFO | `offline_duration_ms` |
| `network.retry.attempt` | DEBUG | `attempt_number`, `max_attempts`, `backoff_ms` |

### Redaction Rules
- Never log full URLs with query parameters that may contain tokens.
- Log only the host portion of URLs (`url_host`), not the full path.
- Never log request or response bodies in network-level logs (module-level logs may log redacted bodies at DEBUG level).

---

## Request Deduplication

For idempotent operations (e.g., update check, license validation):
- If an identical request is already in flight, return the pending result rather than issuing a duplicate request.
- Identity is determined by: method + URL + body hash.
- Non-idempotent operations (POST for activation, admin actions) must never be deduplicated.

---

## Testing Requirements

### Unit Tests
- Timeout enforcement: verify request fails after configured timeout.
- Retry logic: verify correct number of attempts with expected backoff delays.
- Jitter: verify delay is within expected range.
- Non-retryable: verify 4xx responses are not retried.
- Error mapping: verify all network conditions map to correct error codes.
- Deduplication: verify duplicate requests return same result.

### Integration Tests
- Offline → online transition: verify connectivity recovery and event emission.
- Server returning 429 with Retry-After: verify client waits correctly.
- Slow server: verify timeout triggers correctly.
- Certificate error: verify `NET_SSL_ERROR` is raised.

### Security Tests
- Verify no full URLs with tokens appear in logs.
- Verify TLS certificate validation is enforced (no `--insecure` equivalent in production).
