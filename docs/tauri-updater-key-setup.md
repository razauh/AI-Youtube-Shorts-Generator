# Tauri Updater Key Setup

Generate a Tauri updater keypair from the repo root:

```bash
pnpm --dir app run tauri signer generate -- -w app/src-tauri/tauri-updater.key
```

This creates two important values:

- Private key file: `app/src-tauri/tauri-updater.key`
- Public key output/file: usually `app/src-tauri/tauri-updater.key.pub`

Read the public key:

```bash
cat app/src-tauri/tauri-updater.key.pub
```

Put that public key into:

```text
app/src-tauri/tauri.conf.json
```

Replace:

```json
"pubkey": "REPLACE_WITH_TAURI_UPDATER_PUBLIC_KEY"
```

with:

```json
"pubkey": "PASTE_PUBLIC_KEY_HERE"
```

Then add the private key content to this GitHub Actions secret:

```text
TAURI_SIGNING_PRIVATE_KEY
```

Do not commit `app/src-tauri/tauri-updater.key`. It is private.

After adding the public key, run validation manually:

```bash
bash .scripts/run-updater-endpoint-validation.sh
```
