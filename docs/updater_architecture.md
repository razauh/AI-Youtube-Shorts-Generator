# Updater Architecture

## Decision

Decision: retain the Worker updater compatibility endpoint for current customer builds and serve it from a static signed HTTPS manifest source.

The customer Tauri configuration keeps:

`/updates/{{target}}/{{arch}}/{{current_version}}`

The Worker route remains a narrow compatibility proxy. It reads `UPDATE_MANIFEST_URL`, which must be HTTPS, filters the static manifest by target, architecture, and current version, and returns only the Tauri updater fields needed by the signed updater plugin.

## Static Manifest Source

The release pipeline generates `customer-latest.json` from signed customer artifacts. Each platform entry must contain an HTTPS artifact URL and a non-empty updater signature. Tauri still verifies downloaded artifacts with the configured updater public key.

## Why Not Repoint Now

Direct static updater endpoints are simpler long term, but existing customer builds already point at the Worker route. Keeping the Worker path avoids breaking update checks while static hosting is manually validated.

## Rollback

If static manifest hosting fails, keep or restore `UPDATE_MANIFEST_URL` to the last known-good signed manifest. If a future direct static endpoint is adopted and fails, repoint the Tauri updater endpoint back to the Worker `/updates/{{target}}/{{arch}}/{{current_version}}` route.
