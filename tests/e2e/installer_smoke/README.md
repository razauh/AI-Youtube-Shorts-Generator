# Installer Smoke Tests

These scripts inspect already-built installer artifacts and release configuration. They do not build, install dependencies, deploy, upload, or delete user data.

Run from the repository root after producing release artifacts:

```bash
bash tests/e2e/installer_smoke/linux-smoke.sh app/src-tauri/target/release/bundle
bash tests/e2e/installer_smoke/macos-smoke.sh app/src-tauri/target/release/bundle
powershell -ExecutionPolicy Bypass -File tests/e2e/installer_smoke/windows-smoke.ps1 -ArtifactDir app/src-tauri/target/release/bundle
```

Each script creates `.logs/`, writes a timestamped log, and exits non-zero when required artifacts or release configuration are missing.
