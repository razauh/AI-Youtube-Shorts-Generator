# Third-Party Notices

Last updated: May 25, 2026

This document summarizes third-party software, libraries, tools, APIs, models, and services identified from the inspected repository and `repomix-output.xml`. It is not a complete open-source license audit. Exact license texts, binary redistribution obligations, model licenses, and service terms must be verified by a qualified lawyer before commercial release.

Product naming note: the repository and desktop bundle identify the product as "AI YouTube Shorts Generator," while parts of the UI refer to "Signal Forge." [VERIFY: final production product name]

## 1. General Notice

The application may include, depend on, execute, or connect to third-party software, services, APIs, models, datasets, platforms, and tools. Each third-party component remains governed by its own license, notices, terms, policies, and restrictions.

Nothing in this application grants users rights to third-party content, websites, APIs, services, models, datasets, media, music, images, fonts, videos, software, or platform data beyond rights granted by the applicable owner or license.

Users and distributors are responsible for complying with third-party terms, especially when downloading platform content, using API keys, redistributing binaries, bundling FFmpeg/runtime packs, downloading models, or publishing generated outputs.

## 2. Online Services and APIs

The inspected codebase may use or connect to the following services.

### MuAPI

Used in API mode for hosted processing:

- `youtube-download` for source video download by URL and format.
- `openai-whisper` for hosted transcription.
- `gpt-5-mini` for transcript classification and highlight-selection prompts.
- `autocrop` for clip rendering/reframing.
- Polling endpoints under `/predictions/{request_id}/result`.

The app sends MuAPI API keys in `x-api-key` headers and sends processing payloads such as source URLs, hosted media URLs, transcript-derived prompts, language settings, timestamps, and aspect ratios.

License/terms verification required before release: MuAPI terms, data-processing terms, retention, content policy, pricing/rate limits, and attribution requirements.

### OpenAI

Used by local mode through the Python `openai` package for highlight ranking and prompt-based text generation. The inspected Python default model is `gpt-4o-mini` unless overridden.

The app may send transcript samples, timestamped transcript text, prompt instructions, and highlight-generation context to OpenAI.

License/terms verification required before release: OpenAI API terms, privacy/data-use terms, model behavior disclaimers, output-use terms, and any required user disclosures.

### Gumroad

Used by the Cloudflare Worker licensing backend for purchase/license verification. The Worker receives Gumroad webhook form data and verifies sales through the Gumroad sales API using an operator-configured Gumroad access token.

Data may include sale IDs, product IDs, purchaser email, license key returned by Gumroad, refund/dispute signals, and provider sale metadata.

License/terms verification required before release: Gumroad developer/API terms, webhook terms, payment/refund terms, and privacy/data-processing obligations.

### Cloudflare Workers and D1

Used for the hosted licensing backend, including activation, validation, reset requests, reset status, admin review, Gumroad webhook handling, idempotency records, audit events, license records, and device bindings.

License/terms verification required before release: Cloudflare terms, D1 data location/retention/security terms, Workers terms, and data-processing agreement availability.

### YouTube, Google, and Other Source Platforms

The application can process YouTube URLs and local mode can use yt-dlp to download online video sources. API mode can send source URLs to MuAPI for hosted downloading.

The application is not affiliated with or endorsed by YouTube, Google, TikTok, Instagram, Meta, or other platforms unless separately stated in writing. Users must comply with platform terms, copyright rules, downloader restrictions, API terms, and content policies.

### Update, Runtime-Pack, Model-Hosting, and Crash-Report Endpoints

The inspected app includes Tauri updater integration, local runtime-pack download/repair flows, local model downloads through faster-whisper tooling, and optional user-submitted crash reports if a crash-report endpoint is configured.

Release verification required: final update endpoint, updater signing/public key, runtime-pack manifest/asset host, runtime archive license notices, model-hosting provider terms, crash-report provider terms, and retention policy.

## 3. Frontend and Desktop UI Dependencies

The `app/package.json` manifest identifies the following frontend and desktop JavaScript dependencies:

- `@tauri-apps/api`
- `@tauri-apps/plugin-updater`

Development and test dependencies identified:

- `@tauri-apps/cli`
- `@sveltejs/vite-plugin-svelte`
- `svelte`
- `vite`
- `vitest`
- `jsdom`
- `@testing-library/svelte`
- `@testing-library/jest-dom`

License metadata for these packages was not included in the inspected repomix file. Generate and review a final npm license report before release, including transitive dependencies from `app/package-lock.json` if present in the release repository.

## 4. Rust, Tauri, and Native Dependencies

The `app/src-tauri/Cargo.toml` manifest identifies these Rust/Tauri dependencies:

- `tauri`
- `tauri-build`
- `tauri-plugin-updater`
- `serde`
- `serde_json`
- `reqwest` with `rustls-tls`
- `tokio`
- `rfd`
- `sha2`
- `async-trait`
- `license-control-suite` as a local path dependency under `vendor/license-control-suite`

License metadata and transitive crate license data were not included in the inspected repomix file. Generate and review a final Cargo license report before release, including the local `license-control-suite` path dependency and any bundled license notices required by Tauri, Rust crates, and updater artifacts.

## 5. Python Dependencies

The Python manifests identify:

- `requests`
- `python-dotenv`
- `yt-dlp`
- `faster-whisper`
- `openai`
- `opencv-python`
- Optional `torch` for CUDA Whisper use

Runtime validation and bundling scripts also reference or imply transitive/local-model dependencies such as:

- `ctranslate2`
- `huggingface_hub`
- `tokenizers`
- `av`
- `numpy`

License metadata for Python packages and transitive dependencies was not included in the inspected repomix file. Generate and review a final Python license report for both normal and optional local-mode dependencies before release.

## 6. Media Tools

### FFmpeg

The local pipeline invokes FFmpeg for cutting clips, encoding video with `libx264`, encoding audio with AAC, muxing audio, and producing MP4 outputs.

FFmpeg is a third-party project governed by its own licensing. FFmpeg obligations can vary depending on:

- Whether FFmpeg is bundled, downloaded as a runtime pack, or required from the user's system.
- Whether the build is LGPL, GPL, or includes non-free/commercial codec options.
- Whether `libx264`, AAC, hardware encoders, or other codecs are enabled.
- Distribution platform and jurisdiction.

Release verification required: exact FFmpeg binary source, version, build flags, enabled codecs, license mode, source-offer obligations, attribution, and whether FFmpeg is redistributed with the app or runtime pack.

### yt-dlp

The local pipeline uses yt-dlp to download source media by URL. The inspected bundled-runtime folder includes a yt-dlp launcher/wrapper and runtime information referencing yt-dlp.

yt-dlp is third-party software with its own license and terms. Use of yt-dlp may also be restricted by the terms of the websites being accessed. Users are responsible for complying with platform terms and rights restrictions.

Release verification required: exact yt-dlp version, license notice, whether a binary/wrapper is redistributed, update policy, and platform-compliance disclosures.

### Python Runtime and Runtime Packs

The app can use a bundled Python runtime, a downloaded local runtime pack, or development/runtime paths. Runtime packs may include Python, site-packages, yt-dlp, FFmpeg, and bridge code depending on release packaging.

Release verification required: Python license, bundled package licenses, runtime-pack archive notices, checksum/signature policy, model/cache license terms, and operating-system-specific packaging notices.

## 7. AI and Model Dependencies

The application can use faster-whisper for local transcription and supports selectable Whisper model names such as `tiny`, `base`, `small`, `medium`, `large-v3`, `large-v3-turbo`, and English-specific variants.

Model files may be downloaded and cached locally through faster-whisper or related model-hosting tooling. The inspected code stores local model cache files under an app-specific Hugging Face-style cache directory.

Release verification required: exact model source, model license, model card terms, data-use restrictions, attribution requirements, redistribution rights, commercial-use restrictions, and whether model files are bundled or downloaded on demand.

The local reframing implementation uses OpenCV Haar cascade functionality through `opencv-python`; verify OpenCV package and bundled data-file notices before release.

## 8. Backend and Worker Dependencies

The `worker/package.json` manifest identifies a private Worker package without runtime npm dependencies in the inspected file. The Worker is intended to run on Cloudflare Workers and use Cloudflare D1.

The Worker uses platform-provided Web APIs and Cloudflare bindings for request handling, crypto, D1 database access, and fetch calls to Gumroad.

Development/deployment tooling such as Wrangler is referenced in scripts and documentation but not listed as a committed Worker dependency lockfile in the inspected manifest. Verify any development/deployment tooling licenses separately if redistributed or included in release materials.

## 9. Payment and Licensing Components

The application uses a local Rust path dependency named `license-control-suite` for licensing/authentication contracts, desktop persistence, Tauri command wrappers, and Worker client behavior.

The codebase also includes custom Worker logic for Gumroad sale verification, access-token issuance, hashed license-key records, device bindings, reset requests, admin review, idempotency records, and audit events.

Release verification required: license terms for `license-control-suite`, Gumroad terms, Cloudflare terms, token/signing compliance, data-processing agreements, and refund/consumer-law obligations.

## 10. Testing and Development Dependencies

The repository contains tests and scripts that reference:

- Node test runner and Vitest.
- Testing Library for Svelte.
- jsdom.
- Cargo tests and Rust test utilities.
- Python characterization/parity tests.
- Shell scripts for secure npm install, Cargo checks, bundled runtime preparation, and supply-chain triage.

These tools may not be distributed with production builds, but their licenses should be reviewed if included in source releases, binary bundles, CI artifacts, or developer distributions.

## 11. License Metadata Not Fully Discoverable From Repomix

The inspected repomix file did not include complete lockfiles, transitive dependency inventories, full third-party license texts, FFmpeg build metadata, model cards, runtime-pack manifests, or final production service contracts.

Before release, complete all of the following:

- Generate an npm license report for frontend dependencies and transitive packages.
- Generate a Cargo license report for Rust crates and the local path dependency.
- Generate a Python license report for base and local-mode dependencies.
- Verify FFmpeg, yt-dlp, Python runtime, and runtime-pack redistribution obligations.
- Verify faster-whisper and model-file licenses for all selectable models.
- Verify MuAPI, OpenAI, Gumroad, Cloudflare, update host, crash-report provider, model-host, and platform terms.
- Include required license texts, copyright notices, attribution, source offers, and model notices in release artifacts.
- Confirm whether the app's own license permits the planned distribution model.

## 12. User and Distributor Responsibility

Users must comply with applicable third-party terms when using APIs, downloading content, processing platform media, publishing clips, or using generated outputs commercially.

Distributors must verify and satisfy all third-party license obligations before shipping source code, binaries, installers, runtime packs, model files, or bundled media tools.

