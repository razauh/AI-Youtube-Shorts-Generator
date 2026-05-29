# Customer Updater Worker Endpoint

This repo uses the Cloudflare Worker as the public Tauri updater endpoint for the customer desktop app.

## Endpoint

Customer app config:

```text
https://license-worker.YOUR_CLOUDFLARE_SUBDOMAIN.workers.dev/updates/{{target}}/{{arch}}/{{current_version}}
```

Worker route:

```text
GET /updates/:target/:arch/:current_version
```

Supported targets:

- `windows`
- `linux`
- `darwin`

Supported architectures:

- `x86_64`
- `aarch64`
- `i686`
- `armv7`

The updater route is public. It does not require a license key, access token, admin token, D1 lookup, or idempotency key. Tauri's standard updater check does not send the app's license token.

## Response Contract

If no update is available, the Worker returns:

```text
204 No Content
```

The Worker also returns `204` when the updater is not configured, the platform is unsupported, the platform has no release artifact, or the current version is invalid.

If an update is available, the Worker returns raw Tauri updater JSON:

```json
{
  "version": "0.1.1",
  "url": "https://github.com/razauh/AI-Youtube-Shorts-Generator/releases/download/v0.1.1/windows-x86_64-setup.exe",
  "signature": "contents-of-the-tauri-sig-file",
  "notes": "Customer app update 0.1.1",
  "pub_date": "2026-05-28T00:00:00Z"
}
```

This response is intentionally not wrapped in the Worker license envelope. Do not return `{ "ok": true, "data": ... }` for this route.

If the manifest cannot be fetched or is malformed, the Worker returns a safe `503` storage error.

## Manifest Source

The Worker reads this non-secret variable:

```text
UPDATE_MANIFEST_URL=https://github.com/razauh/AI-Youtube-Shorts-Generator/releases/latest/download/customer-latest.json
```

`customer-latest.json` is created by `.scripts/generate-customer-updater-manifest.mjs` during the release workflow. It has this shape:

```json
{
  "version": "0.1.1",
  "notes": "Customer app update 0.1.1",
  "pub_date": "2026-05-28T00:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "url": "https://github.com/razauh/AI-Youtube-Shorts-Generator/releases/download/v0.1.1/windows-x86_64-setup.exe",
      "signature": "contents-of-the-sig-file"
    }
  }
}
```

## Signing Setup

Production auto-updates require a Tauri updater keypair.

- Private key: GitHub Actions secret `TAURI_SIGNING_PRIVATE_KEY`
- Optional private key password: GitHub Actions secret `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
- Public key: `app/src-tauri/tauri.conf.json` at `plugins.updater.pubkey`

Never commit the private key. Only the public key belongs in the app config.

Before shipping, replace these placeholders in `app/src-tauri/tauri.conf.json`:

```text
REPLACE_WITH_TAURI_UPDATER_PUBLIC_KEY
YOUR_CLOUDFLARE_SUBDOMAIN
```

## Release Flow

1. Update the customer app version in `app/src-tauri/tauri.conf.json`.
2. Ensure GitHub Actions secrets include `TAURI_SIGNING_PRIVATE_KEY`.
3. Commit the release changes.
4. Push a tag:

```bash
git tag v0.1.1
git push origin v0.1.1
```

5. GitHub Actions builds customer and admin artifacts for Windows, Linux, and macOS.
6. The release job creates flattened customer updater assets and `customer-latest.json`.
7. GitHub Release assets are published.
8. Customer app users click `Check for Updates`.
9. The app calls the Worker `/updates/...` endpoint.
10. Tauri downloads the update, verifies the signature using the public key, installs it, and asks the user to restart.

## Validation

Agents must not run validation commands in this repo. Run this manually from the repository root:

```bash
bash .scripts/run-updater-endpoint-validation.sh
```

The script writes logs to `.logs/` and exits non-zero on failure.

The validation script intentionally fails while `YOUR_CLOUDFLARE_SUBDOMAIN` or `REPLACE_WITH_TAURI_UPDATER_PUBLIC_KEY` remains in `app/src-tauri/tauri.conf.json`.

## Rollback Notes

The updater only offers a version when the manifest version is greater than the installed version. Do not use this endpoint for downgrades. To stop offering a bad release, publish a newer fixed version and make sure `customer-latest.json` points to that version.
