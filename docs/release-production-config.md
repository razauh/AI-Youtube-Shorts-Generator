# Release Production Configuration

This repository has one non-secret production configuration path for customer releases.

## Public Origins

- Licensing Worker: `https://license-worker.demandscout.workers.dev`
- Customer updater endpoint: `https://license-worker.demandscout.workers.dev/updates/{{target}}/{{arch}}/{{current_version}}`
- Customer updater manifest source for the Worker: `https://github.com/razauh/AI-Youtube-Shorts-Generator/releases/latest/download/customer-latest.json`
- Local runtime-pack manifest: `https://license-worker.demandscout.workers.dev/runtime-pack/manifest.json`

## Secret Inputs

Do not commit these values. Configure them in the appropriate host or release environment:

- `TAURI_SIGNING_PRIVATE_KEY`
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`, when the private key is password-protected
- Worker secrets: `GUMROAD_ACCESS_TOKEN`, `TOKEN_SIGNING_SECRET`, `HASH_PEPPER`, `ADMIN_API_TOKEN`

## Override Precedence

- Desktop licensing reads `LICENSE_WORKER_BASE_URL` when set; otherwise it uses the production Worker origin above.
- Local runtime setup reads `LOCAL_RUNTIME_PACK_MANIFEST_URL` when set; otherwise it uses the production runtime-pack manifest above, unless an app-data `runtime-pack/manifest.json` exists for local repair/testing.
- The updater endpoint and public key are read from `app/src-tauri/tauri.conf.json`.

## Manual Validation

Agents must not run validation commands in this repository. Run these manually from the repository root before a public release:

```bash
bash .scripts/validate-release-ci-config.sh
bash .scripts/run-updater-endpoint-validation.sh
bash .scripts/run-legal-policy-validation.sh
```
