# Third-Party Notices

Last updated: May 23, 2026

This document summarizes third-party components, tools, APIs, models, and services identified during repository review. It is not a complete license audit and should be reviewed by a qualified lawyer before public release.

## 1. General Notice

The application may include, depend on, or connect to third-party software, tools, libraries, APIs, models, datasets, and services. Each third-party component remains governed by its own license, terms, policies, and notices.

Nothing in this repository grants users rights to third-party content, services, APIs, models, datasets, media, music, images, fonts, or software beyond the rights granted by the applicable third-party owner or license.

## 2. APIs and Online Services

The application may use or connect to:

- MuAPI for API-mode download, transcription, highlight processing, LLM-style ranking, autocrop, and media processing.
- OpenAI for local-mode highlight ranking or other AI processing.
- Gumroad for purchase/license verification flows.
- Cloudflare Worker/D1 infrastructure for licensing backend behavior.
- Update endpoints configured for Tauri updater behavior.
- Crash-report endpoints if configured and used.

Users are responsible for complying with each provider's terms, privacy policy, usage limits, billing rules, and content rules.

## 3. FFmpeg

The application may use FFmpeg for media cutting, encoding, muxing, conversion, and output generation.

FFmpeg is a third-party project governed by its own licenses. FFmpeg licensing obligations can vary depending on how FFmpeg is built, configured, linked, bundled, and distributed.

[VERIFY: whether release builds bundle FFmpeg or require external installation]

[VERIFY: FFmpeg build configuration, enabled codecs, and LGPL/GPL/commercial implications for distributed binaries]

## 4. Local Media and AI Dependencies

The application may use local tools and libraries such as Python, yt-dlp, OpenCV, faster-whisper, Whisper-style model files, GPU/CUDA-related packages, and system libraries.

These components have their own licenses and technical requirements. Users and distributors are responsible for complying with applicable licenses and terms.

[VERIFY: exact licenses for bundled or recommended local model files]

[VERIFY: exact licenses for distributed Python/runtime/tool binaries, if any]

## 5. Frontend, Backend, and Runtime Dependencies

The repository includes frontend, Rust/Tauri, Python, Worker, and test dependencies. These dependencies are governed by their own open-source or commercial licenses.

Before public release or redistribution, generate and review a complete dependency license report for:

- Node/npm dependencies.
- Rust crates.
- Python packages.
- Tauri plugins.
- Cloudflare Worker dependencies.
- Bundled runtime tools.
- AI model files.

## 6. User Responsibility

Users are responsible for ensuring their use of third-party services, APIs, tools, models, and content complies with applicable third-party terms and licenses.

The application developers are not responsible for a user's violation of third-party terms, API rules, open-source licenses, model licenses, platform rules, or content rights.

## 7. Missing Final Notices

Before release, add or verify:

- [VERIFY: complete open-source license inventory]
- [VERIFY: included license texts or links for redistributed components]
- [VERIFY: FFmpeg notice and source-offer requirements, if applicable]
- [VERIFY: model license notices]
- [VERIFY: third-party API terms links]
- [VERIFY: app's own license file and copyright notice]
