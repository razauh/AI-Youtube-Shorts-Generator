# Tauri Updater Key Setup

The customer app already has a public updater key in `app/src-tauri/tauri.conf.json`. Generate a replacement keypair only when intentionally rotating release signing keys.

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

Put the replacement public key into:

```text
app/src-tauri/tauri.conf.json
```

Replace the existing `plugins.updater.pubkey` value with the new public key. Keep the existing updater endpoint unless the production Worker origin is deliberately changed.

Then add the private key content to this GitHub Actions secret:

```text
TAURI_SIGNING_PRIVATE_KEY
```

Do not commit `app/src-tauri/tauri-updater.key`. It is private.

After rotating the public key, run validation manually:

```bash
bash .scripts/run-updater-endpoint-validation.sh
```
