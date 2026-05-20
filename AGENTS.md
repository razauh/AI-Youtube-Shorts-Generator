# AGENTS.md

Security and operating rules for AI coding agents working in this repository.

This project contains a Tauri desktop app, a Rust backend, a Cloudflare Worker licensing service, Python legacy pipeline code, local media processing, and purchase/license flows. Treat it as security-sensitive software. When instructions conflict, choose the safer path and ask before taking actions that could weaken security, modify dependencies, expose secrets, or change licensing behavior.

## Non-Negotiable Rules

- Do not run destructive commands such as `rm -rf`, `git reset --hard`, `git checkout --`, or mass file rewrites unless the user explicitly requests them.
- Do not modify dependency manifests, lockfiles, `.npmrc`, Cargo manifests, Python requirement files, CI workflows, release scripts, signing config, updater config, license logic, authentication logic, or secret handling unless the task explicitly asks for that class of change.
- Do not add telemetry, analytics, crash uploads, remote logging, or network calls unless the user explicitly requests them and the implementation has clear consent and redaction.
- Do not print, persist, commit, or include in test snapshots: license keys, access tokens, API keys, private keys, Gumroad secrets, MuAPI keys, OpenAI keys, Wrangler tokens, device identifiers, full machine fingerprints, or raw customer emails except where already required by an existing contract and redacted in logs.
- Never bypass tests by weakening assertions, deleting tests, broadening mocks, or hiding failures. Fix the cause or report the blocker.
- Never commit generated build output, local media output, `node_modules`, Python virtualenvs, logs containing secrets, `.env` files, or private release artifacts.

## Dependency Safety Policy

Normal development uses the existing dependency tree. Dependency changes must be separate, deliberate work.

- Never run `npm install`, `npm update`, `npm add`, `npx`, `npm exec`, `pnpm add`, `yarn add`, `cargo add`, `cargo update`, `pip install` for new packages, or similar dependency-changing commands without explicit user approval.
- For the Svelte/Tauri frontend in `app/`, normal setup is `npm --prefix app ci` only.
- `app/.npmrc` is intentionally strict: `package-lock=true`, `save-exact=true`, `ignore-scripts=true`, `audit=true`, `fund=false`, and `engine-strict=true`. Do not relax these defaults without explicit approval.
- Do not run npm lifecycle scripts during install. If a package genuinely requires install scripts, stop and ask for approval with the exact package, version, script, and reason.
- Do not use `npx` or `npm exec` to fetch transient tooling. Use checked-in scripts or already-installed local binaries.
- Do not modify `package.json`, `package-lock.json`, `.npmrc`, `Cargo.toml`, `Cargo.lock`, `requirements.txt`, or `requirements-local.txt` unless dependency change is the task.
- Any dependency change must show the exact command used, manifest diffs, lockfile diff summary, `npm audit` or equivalent result, and registry signature/audit result where applicable.
- The `worker/` directory currently has `package.json` without a committed lockfile. Do not create or update worker lockfiles as a side effect of unrelated work.
- Prefer existing project scripts: `scripts/secure-npm-install.sh`, `scripts/audit-installed-npm.sh`, and `scripts/secure-cargo-check.sh`.

## Command Execution Guardrails

- Inspect scripts before running them, especially scripts that invoke package managers, shells, deployment tools, Tauri, Wrangler, Cargo build scripts, Python subprocesses, or external services.
- Prefer read-only exploration with `rg`, `sed`, `git diff`, `git status`, and `cargo metadata --locked` style commands.
- Do not start dev servers, Tauri apps, browser GUIs, deployment commands, or long-running network services without a user-facing reason.
- Do not run `wrangler deploy`, Tauri bundle/sign/release commands, updater publication commands, or any command that changes cloud state unless explicitly requested.
- If a test/build command fails because of missing dependencies or blocked network, do not “fix” it by installing packages. Report the blocker or ask for approval.

## Secrets and Environment

- Treat `.env`, shell environment, keyrings, OS credential stores, Wrangler config, Tauri signing keys, updater private keys, and license worker credentials as sensitive.
- Do not read secret files unless the task is explicitly about secret configuration and the user approves.
- Do not echo environment variables or include them in logs.
- Redact secrets in all user-facing output. Use placeholders such as `[redacted]`, `[redacted-license-key]`, or masked license keys.
- Preserve existing crash/debug redaction behavior. Any new support bundle or diagnostic output must redact license keys, tokens, emails where possible, machine IDs, paths that reveal usernames when not needed, and request authorization headers.

## Licensing and Authentication

- License activation is security-sensitive. Keep license-key activation, device binding, session validation, reset request, token storage, and entitlement state changes tightly scoped.
- Do not change Gumroad, license worker, Tauri auth commands, token validation, device fingerprinting, reset approval, or local secure storage behavior unless explicitly requested.
- Never store plaintext license keys in frontend state beyond the active form field, browser local storage, logs, crash drafts, or support exports.
- Never show the main app UI to unauthenticated users unless the task explicitly changes licensing policy.
- Purchaser email is for purchase records, reset, and support flows. Do not turn it into a mandatory login identity without a deliberate backend contract change.
- Preserve command redaction in Rust `Debug` implementations and frontend tests that assert plaintext license material is not persisted.

## Frontend and Tauri UI

- For locked or unauthenticated states, ensure generator, library, legal/help navigation, and any privileged actions are hidden unless intentionally exposed.
- Keep Tauri IPC calls typed and routed through existing client wrappers where possible.
- When adding a new Tauri command, register it in the Rust invoke handler and capabilities, return typed errors, and add tests for command inventory/security where the repo has patterns.
- Do not add browser APIs, file pickers, shell/open operations, or updater operations without checking Tauri permissions/capabilities.
- Avoid adding third-party frontend libraries for UI convenience. Use existing Svelte, CSS, and project patterns.
- Do not weaken Content Security Policy, updater signing, Tauri allowlists/capabilities, or window security settings without explicit approval.

## Backend, Worker, and API Security

- Treat `worker/` as a licensing boundary. Validate inputs, return structured errors, and avoid leaking whether a secret exists beyond established contract behavior.
- Do not log raw license keys, access tokens, Gumroad payload secrets, provider verification tokens, or full purchaser emails.
- Keep idempotency, replay handling, entitlement checks, device binding checks, and reset state transitions intact unless the task explicitly targets them.
- Any new endpoint must define method, path, request shape, response shape, error codes, auth requirements, rate-limit considerations, and tests.
- Webhook handlers must verify provider authenticity before mutating entitlement or license records.
- Do not deploy or mutate Cloudflare D1/Worker state from an agent session unless explicitly requested.

## Rust and Native Code

- Use `cargo check/test --locked` for normal validation. Do not update lockfiles during routine work.
- Do not introduce `unsafe` code. If unavoidable, stop and ask with a concrete justification and safety proof.
- Do not add process execution, filesystem writes outside approved app data/output locations, network clients, cryptography, or key storage changes without explicit scope.
- Prefer existing error types and redaction patterns. Errors crossing Tauri IPC must be safe for display and must not include secrets.
- Keep local persistence, keyring use, protected storage, and updater code conservative and tested.

## Python and Media Pipeline

- Do not install Python packages or modify requirement files unless explicitly requested.
- Do not execute untrusted media URLs/files for tests unless the task requires it and the user approves the source.
- Do not add shell command construction with string interpolation for `ffmpeg`, `yt-dlp`, or subprocess calls. Use argument arrays and validate paths/URLs.
- Treat downloaded videos, transcripts, generated clips, and output JSON as user data. Do not upload or log them outside requested API mode behavior.
- Avoid adding remote LLM/API calls in tests. Use fixtures/mocks.

## Data, Privacy, and Logging

- Minimize data retention. Keep user project history, media metadata, debug logs, and crash drafts local unless explicit upload is requested.
- Do not add automatic cloud sync.
- Do not include full local paths, usernames, machine identifiers, fingerprints, emails, source URLs, transcripts, or generated content in logs unless required for the feature and clearly redacted where appropriate.
- Support/debug exports must be user-initiated and inspectable before sending.
- Keep refund/legal/support copy factual; do not imply automated refunds, account recovery, or support guarantees that are not implemented.

## Testing and Verification

- Run the narrowest relevant tests first, then broader tests when practical.
- For frontend changes, prefer `npm --prefix app test` only after confirming no install is needed and package scripts are safe.
- For Rust/Tauri changes, prefer `cargo test --manifest-path app/src-tauri/Cargo.toml --locked` or the specific test target.
- For licensing changes, include tests for invalid license, device already bound, reset request/status, session reauth, redaction, and no plaintext license persistence.
- For worker contract changes, update fixtures and contract tests together.
- If a command cannot be run safely because of supply-chain, network, sandbox, missing dependency, or secret concerns, say so and provide the exact unrun command.

## Git Hygiene

- Always check `git status --short` before editing and before final response.
- Preserve user changes. If unrelated files are modified, do not revert them.
- Keep dependency changes separate from feature/security/code changes.
- Do not commit unless the user asks.
- Do not stage files unless the user asks.
- In the final response, list only files you changed and mention unrelated dirty files separately if relevant.

## Review Checklist Before Finishing

- No new dependency or lockfile drift unless explicitly requested.
- No secret exposure in code, tests, logs, snapshots, or final answer.
- No lifecycle install scripts enabled.
- Auth/license gates still protect privileged UI and backend actions.
- Inputs are validated and errors are safe to show.
- Tests/builds run, or blockers are clearly reported.
- `git diff` confirms the change is scoped to the task.
