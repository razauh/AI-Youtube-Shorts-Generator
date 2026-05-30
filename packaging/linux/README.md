# Linux Packaging

Release artifacts:
- AppImage
- .deb

Runtime source:
- The customer bundle includes `app/src-tauri/bundled-runtime` when a self-contained runtime is prepared.
- If the bundled runtime is not present, local mode downloads the runtime pack from the production runtime-pack manifest URL configured in Rust.

Smoke validation:

```bash
bash tests/e2e/installer_smoke/linux-smoke.sh app/src-tauri/target/release/bundle
```
