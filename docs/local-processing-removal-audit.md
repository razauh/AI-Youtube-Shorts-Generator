# Local Processing Removal Audit

## 1. Executive Summary

Status: **FAIL**.

The current source no longer shows a working local generation chain from frontend UI through Tauri IPC into a local runtime, local model, local transcription, or local video-processing backend. The active app path is API mode only.

The removal is incomplete because tracked release/runtime packaging scripts, release workflow steps, Worker runtime-pack routes/tests/docs, stale packaging documentation, and one Rust backend `mode == "local"` branch remain. Several ignored or generated artifacts also still contain old local-processing graph/cache/runtime material and should be handled separately from source cleanup.

## 2. Scope

This audit covers current filesystem state for:

- Frontend generation UI and API wrappers under `app/src/`.
- Tauri command registration and Rust generation pipeline under `app/src-tauri/src/`.
- Python legacy pipeline files under `python_legacy/`.
- Release/runtime scripts under `scripts/`, `.scripts/`, and `.github/workflows/`.
- Worker runtime-pack references under `worker/`.
- Documentation under `docs/`, `packaging/`, and `README.md`.
- Existing graphify outputs as historical relationship evidence only.

This audit does not modify application code, tests, configs, dependency manifests, lockfiles, CI workflows, release scripts, licensing logic, generated artifacts, or validation scripts.

## 3. Methodology

Evidence was gathered with read-only inspection only:

- `git status --short` to identify existing dirty worktree state before edits.
- `git ls-files` to distinguish tracked source from deleted/untracked current files.
- `find` to identify graphify/cache/artifact locations.
- `rg --no-ignore` to search for local-processing strings across ignored and tracked locations.
- `nl -ba` with `sed` windows to verify source evidence and line numbers.

Existing graphify outputs were used only to identify historical relationships to verify against the current filesystem. Graphify was not regenerated.

## 4. Graph Evidence

Existing graph caches still contain old local-processing relationships. For example, `python_legacy/graphify-out/graph.json` still describes `_run_local()`, `download_youtube_local()`, `transcribe_local()`, and calls from `generate_shorts()` into `_run_local()`. `tests/graphify-out/graph.json` still records `tests/characterization/run_python_baseline.py` calling `python_legacy.shorts_generator.pipeline.generate_shorts`.

Those graph outputs are stale relative to the current filesystem. Direct inspection shows the local Python bridge files and local-mode Python modules are deleted from the working tree, while graph/cache outputs remain as generated or ignored artifacts.

Graph conclusion: current source no longer shows an active path:

`Frontend UI -> IPC invoke -> Tauri command -> local runtime/model/transcription/video-processing`.

## 5. Current Active Path Assessment

Frontend:

- `app/src/routes/+page.svelte` initializes `mode = 'api'` and submits generation through `runGenerateAndStream()` with the current `mode`.
- `app/src/lib/api/tauriClient.ts` defines `GenerateRequest.mode` as `'api'` only.
- Searches for `pickLocalVideoFile`, `pick_local_video_file`, `openInFileManager`, `open_in_file_manager`, `localModel`, `local_model`, `runtimePack`, and `runtime_pack` in active app/frontend/Tauri source found no local UI command wrapper path.

Tauri:

- `app/src-tauri/src/main.rs` registers generation, auth, privacy, health, runtime context, runtime filesystem, secure store, and API key profile commands, but no local video picker, local runtime-pack, local model, or local bridge commands.
- `app/src-tauri/src/commands/generate.rs` still accepts a string `mode` from IPC, but the live pipeline only has API implementation.

Rust pipeline:

- `app/src-tauri/src/core/pipeline.rs` rejects any normalized mode other than `api`.
- `generate_shorts_with_progress_live()` still has a `request.mode.to_lowercase() == "local"` branch, but it passes the same request into the API-only guard, so it is a dead/broken branch rather than a working local path.

Python:

- The current filesystem no longer contains `python_legacy/bridge_entry.py` or `python_legacy/shorts_generator/local/*.py`.
- The remaining Python legacy code is API-mode oriented and includes MuAPI/OpenAI Whisper references that are outside this local-processing removal audit unless they point to local runtime/model/transcription.

## 6. Confirmed Removals

Direct current-source searches and file inspection confirm absence of:

- Local mode UI controls and local file picker submission flow.
- Frontend local runtime/model IPC wrappers.
- Tauri command registrations for local runtime-pack setup, local model profiles, local model status/download, local file picking, or OS file manager open commands.
- Rust `core/local_mode` source files in the current filesystem.
- Rust `runtime/python_runtime.rs`, `runtime/process_supervisor.rs`, and `runtime/tool_resolver.rs` in the current filesystem.
- Python local bridge entry and Python local pipeline modules in the current filesystem.
- Local-processing Rust integration tests in the current filesystem.

Important nuance: many of these paths are still tracked in Git but appear as deleted in the current dirty worktree. This report audits the current filesystem state, not a committed baseline.

## 7. Findings Table

| File path | Classification | Symbol/string | Evidence | Impact | Confidence | Recommended action |
| --- | --- | --- | --- | --- | --- | --- |
| `app/src-tauri/src/core/pipeline.rs` | Dead/broken backend remnant | `request.mode.to_lowercase() == "local"` | Lines 472-475 route `local` into `MockPipelineStages`, while lines 111-118 reject non-`api` modes. | Confusing backend behavior; local mode returns an unknown-mode error instead of being fully removed or explicitly unsupported. | High | Remove the branch or replace it with an explicit safe unsupported-mode error at the boundary. |
| `.github/workflows/release.yml` | Tracked CI/release remnant | `build-bundled-runtime-*`, `prepare-bundled-runtime.sh`, `scan-bundled-runtime.sh` | Lines 72-89 and 185-202 still build and stage bundled runtime inputs. | Release workflow may continue trying to package removed local runtime assets. | High | Remove or revise bundled-runtime steps in a separate release/CI cleanup. |
| `scripts/build-bundled-runtime-unix.sh` | Tracked packaging remnant | `requirements-local.txt`, `python_legacy`, `faster_whisper`, `yt_dlp`, `ffmpeg` | Lines 72-77 install local requirements and copy `python_legacy`; lines 102-107 validate local media/model modules. | Script is inconsistent with removed local runtime source and may fail or recreate local-runtime payloads. | High | Delete or repurpose after deciding whether runtime-pack delivery is still needed. |
| `scripts/build-bundled-runtime-windows.ps1` | Tracked packaging remnant | `python_legacy`, `yt-dlp.exe`, `faster_whisper` | Lines 83-87 copy `python_legacy`; lines 89-107 download/validate local runtime dependencies. | Windows release path still expects local runtime packaging. | High | Delete or repurpose in release cleanup. |
| `scripts/prepare-bundled-runtime.sh` | Tracked packaging remnant | `app/src-tauri/bundled-runtime`, `python_legacy/bridge_entry.py` | Lines 4-11 document bundled runtime layout; lines 21-22 stage into `app/src-tauri/bundled-runtime`; lines 54-56 require `bridge_entry.py`. | Staging script still enforces a deleted local bridge artifact. | High | Remove or replace with current release artifact validation. |
| `.scripts/scan-bundled-runtime.sh` | Tracked validation remnant | `app/src-tauri/bundled-runtime` | Lines 4-15 scan the bundled runtime directory. | Validation still treats bundled local runtime as release surface. | Medium | Remove from release validation if local runtime packaging is retired. |
| `.scripts/validate-release-ci-config.sh` | Tracked validation remnant | bundled runtime builder variables and checks | Lines 35-38 define runtime builder/prepare/scan scripts; lines 62-75 require them. | Release validation can keep obsolete runtime-pack steps alive. | High | Update after CI/release policy decision. |
| `worker/src/index.js` | Worker runtime-pack remnant | `/runtime-pack/manifest.json` | Lines 50-52 route the runtime-pack manifest. | Worker still exposes runtime-pack delivery even though desktop local runtime path is gone. | High | Remove route or document it as intentionally retained for another consumer. |
| `worker/test/contract.test.js` | Worker contract remnant | runtime-pack manifest tests | Lines 486-509 test unconfigured and proxied runtime-pack manifest behavior. | Contract tests preserve runtime-pack API behavior. | High | Remove/update tests with Worker route cleanup. |
| `worker/README.md` | Worker docs remnant | `GET /runtime-pack/manifest.json` | Lines 10-15 list the route; lines 60-61 document production runtime-pack setup default. | Public docs advertise obsolete runtime-pack behavior. | High | Update with Worker route cleanup. |
| `docs/local-processing-components.md` | Stale/untracked documentation | local-processing component inventory | Lines 1-80 document local UI, local IPC wrappers, local bridge, Python local modules, and local model/runtime-pack setup. | Misleads maintainers; contradicts current filesystem state. | High | Delete or replace with this removal audit. |
| `packaging/linux/README.md`, `packaging/macos/README.md`, `packaging/windows/README.md` | Tracked packaging docs remnant | local mode runtime-pack copy | Linux lines 7-9 state local mode downloads runtime pack if bundled runtime is absent; macOS/Windows have equivalent text. | Installer docs still describe removed local mode. | High | Remove local runtime-pack claims from packaging docs. |
| `README.md` | Tracked docs remnant | `python_legacy/ Python CLI and API-mode pipeline` | Search result at line 142 still lists Python legacy pipeline. | Not necessarily local mode, but should be reviewed after Python-local deletion. | Medium | Update only if Python legacy scope is now API-mode only or deprecated. |
| `app/src-tauri/bundled-runtime/`, `python_legacy/graphify-out/`, `graphify-out/`, `tests/graphify-out/`, `app/dist/`, `target/debug/bundled-runtime/`, `__pycache__/` | Generated/ignored artifact remnants | stale local bridge/runtime graph/cache/build files | `find` and `rg --no-ignore` show graph caches, pyc files, and bundled-runtime artifacts still contain old local-processing references. | Can confuse audits and broad searches, but should not be mixed with source cleanup. | Medium | Clean generated/ignored artifacts separately after user approval. |

## 8. Tracked Cleanup Checklist

- Remove or replace the dead `local` branch in `app/src-tauri/src/core/pipeline.rs`.
- Remove bundled-runtime build/stage/scan steps from `.github/workflows/release.yml`, or explicitly document why runtime-pack delivery remains.
- Remove or repurpose `scripts/build-bundled-runtime-unix.sh`.
- Remove or repurpose `scripts/build-bundled-runtime-windows.ps1`.
- Remove or repurpose `scripts/prepare-bundled-runtime.sh`.
- Remove or repurpose `.scripts/scan-bundled-runtime.sh`.
- Update `.scripts/validate-release-ci-config.sh` so it no longer requires bundled-runtime scripts if local runtime packaging is retired.
- Remove or intentionally retain `/runtime-pack/manifest.json` in `worker/src/index.js`.
- Update `worker/test/contract.test.js` and `worker/README.md` together with the Worker runtime-pack route decision.
- Update packaging READMEs to remove local mode/runtime-pack claims.
- Review `README.md` for current Python legacy scope.
- Delete or replace stale `docs/local-processing-components.md` if it remains in the working tree.

## 9. Generated And Ignored Artifact Notes

The following should be treated as generated/ignored cleanup, not source cleanup:

- `graphify-out/` and scoped graphify caches.
- `python_legacy/graphify-out/`.
- `tests/graphify-out/`.
- `app/src-tauri/bundled-runtime/python_legacy/graphify-out/`.
- `app/src-tauri/bundled-runtime/**/__pycache__/`.
- `python_legacy/**/__pycache__/`.
- `app/dist/`.
- `target/debug/bundled-runtime/`.

Do not remove these with destructive commands from an agent session unless the user explicitly asks for that cleanup.

## 10. Exclusions

The audit intentionally excludes API-mode MuAPI and OpenAI Whisper references unless they directly indicate local runtime/model/transcription. Examples excluded from local-processing findings:

- `app/src-tauri/src/core/api_mode/transcriber.rs` references to `openai-whisper`.
- `python_legacy/shorts_generator/transcriber.py` references to MuAPI `/openai-whisper`.
- Test references to local HTTP servers, local storage, local license session handling, or "local/private worker" health labels.

These are not evidence of a local media-processing runtime path.

## 11. Verification Log

Commands used for verification were read-only:

- `git status --short`
- `rg --files`
- `find . -maxdepth 3 -type f -name '*graph*' -o -path './graphify-out/*'`
- `git ls-files ...`
- `rg --no-ignore ...`
- `nl -ba ...`
- `sed` line windows via `nl -ba file | sed -n 'start,endp'`

Commands intentionally not run:

- No tests, builds, linters, formatters, dev servers, validation scripts, dependency installs, deployments, graphify regeneration, or destructive cleanup commands.

No validation script was created because this is a documentation-only audit report.

## 12. Conclusion

The local-processing feature is removed from the active app path, but the removal is incomplete. The highest-priority cleanup is to remove or explicitly retire the backend `local` branch, bundled runtime packaging/release references, Worker runtime-pack contract, and stale documentation. Generated graph/cache/runtime artifacts should be cleaned only as a separate approved artifact cleanup step.
