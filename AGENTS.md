# AGENTS.md

Security and operating rules for AI coding agents working in this repository.

This project contains a Tauri desktop app, a Rust backend, a Cloudflare Worker licensing service, Python legacy pipeline code, local media processing, and purchase/license flows. Treat it as security-sensitive software. When instructions conflict, choose the safer path and ask before taking actions that could weaken security, modify dependencies, expose secrets, or change licensing behavior.

## Decision Ambiguity

- When there is confusion or uncertainty in decision making, do not assume the intended path on your own.
- Present the user with clear, practical options in plain English and ask them to choose before proceeding.
- Each option should briefly explain what will happen if that option is selected.

## Non-Negotiable Rules

- Do not run destructive commands such as `rm -rf`, `git reset --hard`, `git checkout --`, or mass file rewrites unless the user explicitly requests them.
- Do not modify dependency manifests, lockfiles, `.npmrc`, Cargo manifests, Python requirement files, CI workflows, release scripts, signing config, updater config, license logic, authentication logic, or secret handling unless the task explicitly asks for that class of change.
- Do not add telemetry, analytics, crash uploads, remote logging, or network calls unless the user explicitly requests them and the implementation has clear consent and redaction.
- Do not print, persist, commit, or include in test snapshots: license keys, access tokens, API keys, private keys, Gumroad secrets, MuAPI keys, OpenAI keys, Wrangler tokens, device identifiers, full machine fingerprints, or raw customer emails except where already required by an existing contract and redacted in logs.
- Never bypass tests by weakening assertions, deleting tests, broadening mocks, or hiding failures. Fix the cause or report the blocker.
- Never commit generated build output, local media output, `node_modules`, Python virtualenvs, logs containing secrets, `.env` files, or private release artifacts.

## Dependency Safety Policy

Normal development uses the existing dependency tree. Dependency changes must be separate, deliberate work.

- If the JavaScript project has just been initiated or no package-manager policy exists yet, do not default to npm. Propose a secure pnpm setup first and ask the user to approve before creating manifests or running install commands.
- A new JavaScript workspace should use pnpm with a pinned `packageManager`, a committed `pnpm-workspace.yaml`, a committed `pnpm-lock.yaml`, and no npm lockfiles.
- The initial pnpm policy should include: `minimumReleaseAge=43200` minutes, `minimumReleaseAgeStrict=true`, `minimumReleaseAgeIgnoreMissingTime=false`, `ignoreScripts=true`, `strictDepBuilds=true`, `dangerouslyAllowAllBuilds=false`, `engineStrict=true`, and `savePrefix=''`.
- For new projects, add safe package scripts for normal locked install, optional local Linux bubblewrap sandbox install, installed-tree audit, and project validation before suggesting dependency changes.
- For new projects, create `.gitignore` entries for `node_modules`, build output, logs, env files, local caches, and package-manager debug logs before running any dependency setup.
- For new projects, document the exact manual setup commands for the user: `corepack enable`, `pnpm install`, optional `pnpm run deps:sandbox-install`, and the project validation script.
- Do not create `package-lock.json`, `npm-shrinkwrap.json`, npm-only `.npmrc` policy, or npm-based CI for a new project unless the user explicitly chooses npm after being shown the safer pnpm option.
- If an npm-based project already exists, do not silently migrate it. Present clear options in plain English: keep npm and harden it, migrate to pnpm, or pause for a dependency/security review.
- Never run `npm install`, `npm update`, `npm add`, `npx`, `npm exec`, `pnpm install`, `pnpm update`, `pnpm add`, `pnpm dlx`, `pnpm exec`, `yarn add`, `cargo add`, `cargo update`, `pip install` for new packages, or similar dependency-changing commands without explicit user approval.
- For JavaScript workspaces, normal setup is `pnpm install --frozen-lockfile` from the repository root only.
- `pnpm-workspace.yaml` is intentionally strict: `minimumReleaseAge=43200` minutes, `minimumReleaseAgeStrict=true`, `minimumReleaseAgeIgnoreMissingTime=false`, `ignoreScripts=true`, `strictDepBuilds=true`, `dangerouslyAllowAllBuilds=false`, `engineStrict=true`, and `savePrefix=''`. Do not relax these defaults without explicit approval.
- Do not run package lifecycle scripts during install. If a package genuinely requires install scripts, stop and ask for approval with the exact package, version, script, and reason.
- Do not use `npx`, `npm exec`, `pnpm dlx`, or `pnpm exec` to fetch transient tooling. Use checked-in scripts or already-installed local binaries.
- Do not modify `package.json`, `pnpm-lock.yaml`, `pnpm-workspace.yaml`, `.npmrc`, Cargo manifests, `Cargo.lock`, `requirements.txt`, or `requirements-local.txt` unless dependency change is the task.
- Any dependency change must show the exact command used, manifest diffs, lockfile diff summary, `pnpm audit` or equivalent result, and registry signature/audit result where applicable.
- Prefer existing project scripts: `scripts/secure-pnpm-install.sh`, `scripts/secure-pnpm-install-bwrap.sh` for local Linux sandboxed installs, `scripts/audit-installed-pnpm.sh`, and `scripts/secure-cargo-check.sh`.
- Bubblewrap sandboxing is local/manual only. Do not add `bwrap` or `bubblewrap` to GitHub release workflows, updater publication, Tauri signing, Worker deployment, or release manifest generation unless explicitly requested.

## npm Guardrails

- Treat npm as blocked-by-default for this repository. Do not introduce npm commands, npm lockfiles, npm CI steps, npm audit scripts, npm lifecycle-script policy, or npm install documentation unless the task explicitly requires npm.
- Do not run `npm install`, `npm ci`, `npm update`, `npm audit fix`, `npm fund`, `npm publish`, `npm pack`, `npm init`, `npm create`, `npm exec`, or `npx` from agent sessions.
- Do not replace pnpm commands with npm commands to work around pnpm failures. Report the blocker and the exact pnpm command the user should run or approve.
- Do not commit or generate `package-lock.json` or `npm-shrinkwrap.json`. If either appears after a tool run, stop and ask before removing it unless the user explicitly requested npm.
- Do not use npm lifecycle scripts as an escape hatch. Dependency install scripts must stay disabled unless the user approves the specific package, version, script name, script contents, and reason.
- Do not use `npx` for one-off tools, scaffolding, code generation, formatters, test runners, or framework CLIs. Use checked-in scripts, local dependencies already in the lockfile, or ask the user to choose a safe option.
- CI must use `corepack enable`, pinned pnpm, `pnpm install --frozen-lockfile`, and pnpm audit/list commands. Do not add `actions/setup-node` npm cache settings or `npm ci` steps.
- If a third-party guide says to use npm, translate the command to the project’s pnpm equivalent when safe. If the translation is uncertain, ask with clear plain-English options.

## Command Execution Guardrails

- Inspect scripts before running them, especially scripts that invoke package managers, shells, deployment tools, Tauri, Wrangler, Cargo build scripts, Python subprocesses, or external services.
- Prefer read-only exploration with `rg`, `sed`, `git diff`, `git status`, and `cargo metadata --locked` style commands.
- Do not start dev servers, Tauri apps, browser GUIs, deployment commands, or long-running network services without a user-facing reason.
- Do not run `wrangler deploy`, Tauri bundle/sign/release commands, updater publication commands, or any command that changes cloud state unless explicitly requested.
- If a test/build command fails because of missing dependencies or blocked network, do not “fix” it by installing packages. Report the blocker or ask for approval.

## Strict Toolchain Command Policy

- Agents must not run project toolchain validation commands. This includes but is not limited to: `cargo test`, `cargo check`, `cargo build`, `cargo clippy`, `cargo fmt --check`, `npm test`, `npm run test`, `npm run build`, `npm run lint`, `npm run check`, `pnpm test`, `pnpm build`, `pnpm lint`, `pnpm run check`, `yarn test`, `yarn build`, `yarn lint`, `pytest`, `python -m pytest`, and any equivalent install/build/lint/format-check/test/validation command.
- Agents must not run dependency installation commands. This includes but is not limited to: `pip install`, `python -m pip install`, `npm install`, `pnpm install`, and `yarn install`.
- Agents may write or update tests, but must not run those tests.
- Agents must not run validation scripts themselves.
- If a test/check/install/build/validation step is needed, agents must tell the user the exact command(s) to run manually.

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

- Do not run tests/checks/builds/lint/format-check commands directly in agent sessions.
- Do not run project validation scripts directly in agent sessions.
- Provide exact manual commands for the user to run when verification is required.
- For licensing changes, include tests for invalid license, device already bound, reset request/status, session reauth, redaction, and no plaintext license persistence.
- For worker contract changes, update fixtures and contract tests together.
- If a command cannot be run safely because of supply-chain, network, sandbox, missing dependency, or secret concerns, say so and provide the exact unrun command.

## Validation Script Requirement

- When an agent changes code and/or writes tests, it must also add or update a validation bash script under `.scripts/` (for example `.scripts/run-validation.sh`) that covers the relevant checks for the task.
- The script must be safe to run from the repository root.
- The script must create `.logs/` if it does not exist.
- The script must write stdout/stderr logs into `.logs/` using timestamped log filenames where practical.
- The script must exit non-zero if any check fails.
- The script must not install dependencies.
- The script must not delete user data, model caches, generated outputs, or logs.
- The agent must not run the validation script; it must instruct the user how to run it manually.
- For documentation-only changes where no code/tests are changed, do not create a validation script unless explicitly requested.

## Git Hygiene

- Always check `git status --short` before editing and before final response.
- Preserve user changes. If unrelated files are modified, do not revert them.
- Keep dependency changes separate from feature/security/code changes.
- Do not commit unless the user asks.
- Do not stage files unless the user asks.
- In the final response, list only files you changed and mention unrelated dirty files separately if relevant.

## Scope and Safety Boundaries

- Do not modify unrelated files.
- Do not change app behavior unless required by the task.
- Do not remove or weaken existing safety, security, or validation rules.
- Keep changes minimal and directly related to the requested task.
- Do not expose secrets, API keys, tokens, or sensitive local paths in logs or user-facing output.

## Final Response Rules

- After completing a task, provide only a very brief final summary in short bullet points. Do not produce a detailed report, long technical explanation, implementation diary, or full analysis unless the user explicitly requests it.
- Do not include long analysis, long reasoning, implementation diary content, or large technical summaries by default.
- Final responses must be short bullet points only.
- Include only essential completion information:
- Files changed or created.
- What was fixed or added.
- Tests written but not run.
- Validation script path, if created.
- Exact command(s) the user should run, if needed.
- Confirmation that prohibited commands were not run.
- Any critical manual steps the user must perform.
- Avoid long paragraphs, tables, detailed reports, or full reasoning traces unless explicitly requested by the user.

## Review Checklist Before Finishing

- No new dependency or lockfile drift unless explicitly requested.
- No secret exposure in code, tests, logs, snapshots, or final answer.
- No lifecycle install scripts enabled.
- Auth/license gates still protect privileged UI and backend actions.
- Inputs are validated and errors are safe to show.
- Tests/builds run, or blockers are clearly reported.
- `git diff` confirms the change is scoped to the task.
