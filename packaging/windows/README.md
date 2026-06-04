# Windows Packaging

Release artifact:
- MSI

Runtime source:
- Local runtime/model bundles are not packaged for the API-only release path.

Smoke validation:

```powershell
powershell -ExecutionPolicy Bypass -File tests/e2e/installer_smoke/windows-smoke.ps1 -ArtifactDir app/src-tauri/target/release/bundle
```
