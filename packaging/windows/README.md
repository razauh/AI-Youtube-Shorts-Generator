# Windows Packaging

Release artifact:
- MSI

Runtime source:
- The customer bundle includes `app/src-tauri/bundled-runtime` when a self-contained runtime is prepared.
- If the bundled runtime is not present, local mode downloads the runtime pack from the production runtime-pack manifest URL configured in Rust.

Smoke validation:

```powershell
powershell -ExecutionPolicy Bypass -File tests/e2e/installer_smoke/windows-smoke.ps1 -ArtifactDir app/src-tauri/target/release/bundle
```
