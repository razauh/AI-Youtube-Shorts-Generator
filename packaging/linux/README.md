# Linux Packaging

Release artifacts:
- AppImage
- .deb

Runtime source:
- Local runtime/model bundles are not packaged for the API-only release path.

Smoke validation:

```bash
bash tests/e2e/installer_smoke/linux-smoke.sh app/src-tauri/target/release/bundle
```
