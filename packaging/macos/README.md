# macOS Packaging

Release artifacts:
- signed and notarized .app bundle
- DMG or app archive

Runtime source:
- Local runtime/model bundles are not packaged for the API-only release path.

Smoke validation:

```bash
bash tests/e2e/installer_smoke/macos-smoke.sh app/src-tauri/target/release/bundle
```

Run the smoke script on macOS to allow `spctl` notarization assessment.
