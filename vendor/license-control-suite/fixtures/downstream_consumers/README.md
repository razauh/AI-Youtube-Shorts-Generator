# Downstream Consumer Harnesses

These fixtures are intentionally separate from the crate's own `src/` tree and from the example shells under `examples/`.

They exist to answer a different question than unit or integration tests:

- can a downstream Rust consumer depend on the crate through the curated public API?
- can a downstream desktop host wire the Tauri command surface without reaching into internal modules?
- can a downstream crate consume the package through a git dependency instead of a path dependency?
- does the packaged crate still work when consumed outside this repository layout?

## Harnesses

### `core_only_consumer`

Minimal downstream crate that depends on `license-control-suite` with:

- `default-features = false`
- `features = ["core"]`

It uses only `license_control_suite::core` and local fake implementations to demonstrate trait-driven integration.

### `tauri_host_consumer`

Minimal downstream host crate that depends on `license-control-suite` with:

- `features = ["core", "desktop-tauri", "desktop-persistence"]`

It demonstrates host-owned Tauri wiring through `license_control_suite::desktop::tauri`.

### `packaged-consumer`

This is validated by the separate verification script rather than a checked-in crate directory. The script:

1. runs `cargo package`
2. extracts the produced `.crate`
3. creates a temporary consumer crate
4. points that consumer at the extracted packaged source
5. runs `cargo check`

That catches publish-time file and manifest issues that path-based local consumption can miss.

### `git-dependency consumer`

This fixture is stored as a template because the verification script fills in:

- the local git mirror URL
- the current committed revision

The script creates a bare clone of the repository and then checks that a downstream crate can resolve:

- `git = "..."`
- `rev = "..."`

against the committed repository state.

## Manual Verification

Run:

```bash
bash scripts/verify_downstream_consumers.sh
```

Note: the git-dependency check validates committed git state. Uncommitted local edits are not part of that flow.
