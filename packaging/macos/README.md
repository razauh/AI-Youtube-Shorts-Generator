# macOS Packaging

Release artifacts:
- signed and notarized .app bundle
- DMG or app archive

Runtime source:
- The customer bundle includes `app/src-tauri/bundled-runtime` when a self-contained runtime is prepared.
- If the bundled runtime is not present, local mode downloads the runtime pack from the production runtime-pack manifest URL configured in Rust.

Smoke validation:

```bash
bash tests/e2e/installer_smoke/macos-smoke.sh app/src-tauri/target/release/bundle
```

Run the smoke script on macOS to allow `spctl` notarization assessment.
