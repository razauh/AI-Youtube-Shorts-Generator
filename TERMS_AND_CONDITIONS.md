# Terms and Conditions

Last updated: May 23, 2026

These Terms and Conditions ("Terms") apply to your use of [APP NAME]. Repository evidence refers to the project as "AI YouTube Shorts Generator" and some application UI references "Signal Forge." [VERIFY: production app name and legal product name]

[APP NAME] is provided by [DEVELOPER NAME]. Contact: [CONTACT EMAIL]. Governing law: [JURISDICTION].

This document is a practical policy draft for the application and should be reviewed by a qualified lawyer before public release.

## 1. Acceptance of Terms

By installing, activating, accessing, or using the application, you agree to these Terms. If you do not agree, do not use the application.

If you use the application on behalf of a company or organization, you represent that you have authority to accept these Terms on its behalf.

## 2. Description of the Application

The application is a desktop tool for generating short-form video clips from user-provided sources such as YouTube URLs and local video files. Depending on mode and configuration, the application may download or process media, transcribe audio, identify highlight candidates, generate titles/hooks/scores/reasons, create vertical clips, export JSON, store local project history, run dependency checks, and use license activation or validation services.

The application is a tool. It does not guarantee perfect results, lawful outputs, platform acceptance, monetization, engagement, ranking, visibility, or business results.

You are responsible for reviewing and verifying all generated outputs before publishing, sharing, uploading, selling, relying on, or distributing them.

## 3. User Responsibilities

You are responsible for your use of the application and for all content you upload, import, process, generate, export, publish, or distribute.

You are responsible for:

- Ensuring you have rights and permissions for source media and generated outputs.
- Complying with copyright, licensing, privacy, publicity rights, platform rules, API provider terms, and applicable law.
- Reviewing generated video, transcripts, titles, hooks, metadata, scores, and JSON output before use.
- Maintaining valid API keys, credentials, license keys, accounts, dependencies, local files, and configurations.
- Backing up your source files, generated files, projects, configuration, and outputs.

The application does not provide legal, copyright, platform-policy, business, or professional advice.

## 4. Third-Party APIs and Services

The application may use or connect to third-party APIs and services, including MuAPI, OpenAI, Gumroad, Cloudflare Worker services, update services, crash-report endpoints if configured, and any other provider you connect to the application.

Third-party APIs and services are controlled by their respective providers. We do not control their availability, pricing, rate limits, account policies, content policies, output quality, security practices, privacy practices, or terms.

We are not responsible for:

- API downtime, interruptions, latency, or outages.
- Pricing changes, billing disputes, unexpected API costs, or usage charges.
- Rate limits, quota exhaustion, account suspensions, account bans, or provider restrictions.
- Provider policy changes or rejected requests.
- Incorrect, incomplete, unsafe, or delayed API responses.
- Expired, revoked, leaked, restricted, invalid, or misconfigured API keys.
- Losses caused by third-party services or credentials.

You are responsible for complying with all provider terms, policies, usage limits, billing requirements, and content rules.

## 5. API Mode and MuAPI Processing

In API mode, the application may send processing inputs to MuAPI. These inputs may include source URLs, media references, transcript-related data, highlight data, prompt-like processing data, timing data, aspect-ratio settings, and other information needed to generate clips.

MuAPI may perform download, transcription, highlight processing, LLM-based ranking, autocrop, and media-rendering tasks. MuAPI is a third-party service, and its own terms and policies apply.

We are not responsible for MuAPI downtime, processing failures, incorrect results, hosted output availability, rate limits, account restrictions, billing, policy changes, or media-processing errors.

## 6. OpenAI and AI Processing

The application may use OpenAI or other AI services for highlight ranking, text generation, title generation, hook generation, scoring, classification, or similar tasks. In the inspected implementation, local mode still may use OpenAI for the LLM highlight-ranking step.

Information sent to AI providers may include transcript text, prompt text, source-related metadata, highlight-selection instructions, and other processing context.

AI-generated outputs may be inaccurate, incomplete, biased, offensive, misleading, unexpected, duplicative, low quality, legally risky, or unsuitable for your intended use.

You must verify all AI-generated outputs before relying on them or publishing them. We are not responsible for decisions, publications, losses, claims, takedowns, account penalties, monetization loss, or other consequences caused by AI-generated outputs.

## 7. Local AI Models and Local Processing

The application may support local model processing, including faster-whisper or Whisper-style transcription models. Local processing performance depends on your hardware, operating system, model files, GPU drivers, CPU/GPU capability, memory, storage, installed dependencies, permissions, and configuration.

We are not responsible for crashes, slow performance, failed processing, incorrect results, poor transcription quality, missing model files, corrupted model files, unsupported hardware, incompatible drivers, dependency conflicts, or unsupported local environments.

[VERIFY: exact source, hosting provider, and license terms for each selectable local model]

## 8. FFmpeg and Media Processing

The application may use FFmpeg and similar media-processing tools for cutting, encoding, muxing, converting, reframing, and generating media files. FFmpeg is a third-party tool governed by its own license and behavior.

[VERIFY: whether production releases bundle FFmpeg or require users to install it externally]

[VERIFY: FFmpeg build configuration and license obligations for any distributed binaries]

We are not responsible for:

- FFmpeg errors or missing FFmpeg installations.
- Missing codecs or unsupported input/output formats.
- Failed downloads, cuts, conversions, encodes, muxes, or exports.
- Corrupted outputs, partial outputs, quality loss, encoding artifacts, large file sizes, or audio/video desync.
- Incorrect crops, poor face tracking, failed reframing, or unsuitable visual results.
- Hardware acceleration problems, driver problems, or platform-specific media failures.

You are responsible for checking all generated media files before publishing or distributing them.

## 9. User Content and Intellectual Property

You retain whatever rights you already have in your own content. The application does not grant you rights to third-party videos, music, images, likenesses, voices, transcripts, datasets, models, fonts, APIs, software, or other protected material.

You must only process content that you own or have permission to use. You are responsible for obtaining all required rights, permissions, licenses, consents, and clearances.

You are responsible for copyright, licensing, privacy, publicity rights, creator rights, music rights, platform policies, and applicable law.

We are not responsible for copyright claims, DMCA notices, takedowns, rejected uploads, demonetization, lost revenue, account strikes, account bans, platform enforcement, legal claims, or disputes caused by your source content, generated outputs, uploads, or publications.

## 10. Diagnostics, Paths, and Advanced Settings

The application may provide diagnostics, dependency checks, API key profile settings, local model settings, output path controls, local file selection, crash draft handling, and advanced runtime information.

Some settings and paths affect whether the application works correctly. If you manually change dependency paths, executable paths, FFmpeg paths, Python paths, model paths, output paths, working directories, environment variables, permissions, configuration files, diagnostics values, local files, or related settings, you are responsible for the consequences.

We are not responsible if the application stops working, loses access to files, fails dependency checks, fails to process media, produces errors, or generates incorrect outputs because you changed advanced settings, paths, dependencies, permissions, environment variables, or local files.

You should only change advanced settings if you understand the consequences. Reset, retry, recheck, or default options, if available, do not guarantee full recovery.

## 11. Dependencies and Local Environment

The application may require external or bundled dependencies such as FFmpeg, Python, yt-dlp, OpenCV, faster-whisper, AI model files, GPU drivers, system libraries, runtime packages, platform-specific tools, network access, and provider accounts.

Unless a dependency is explicitly bundled with your specific application build, you are responsible for installing, configuring, updating, and maintaining it.

We are not responsible for unsupported operating systems, outdated drivers, antivirus or security software blocking files or processes, missing permissions, corrupted installs, conflicting software, missing runtime packages, dependency version conflicts, incompatible hardware, insufficient CPU/GPU/RAM/disk/network resources, or broken PATH/environment configuration.

## 12. Logs and Diagnostic Information

Logs, crash drafts, diagnostics, output JSON, error messages, and support materials may include sensitive or identifying information, including file paths, dependency paths, local usernames embedded in paths, filenames, source URLs, prompts, transcripts, generated outputs, API errors, environment details, configuration values, stack traces, or processing status.

You should review logs, diagnostics, screenshots, output JSON, and support bundles before sharing them publicly or with support.

We are not responsible if you disclose sensitive information by sharing logs, diagnostics, screenshots, output files, generated JSON, crash drafts, or support materials.

## 13. Privacy and Data Handling

Based on inspected repository behavior, the application supports both local processing and API-based processing. Local files and generated outputs may remain on your device unless you select a workflow that uses external APIs, license services, update services, or configured crash-report submission.

External transmission may occur when:

- You use MuAPI/API mode.
- Local mode sends transcript or prompt context to OpenAI or another configured AI provider.
- You activate, validate, or reset a license.
- The licensing worker verifies a Gumroad purchase.
- You check for or install updates.
- You submit a crash report and a crash-report endpoint is configured.

The application may store local project history, output metadata, settings, local model profile metadata, API key profile metadata, crash drafts, reset status, generated outputs, logs, configuration files, model caches, and license/session/device-related information.

API key values are intended to be stored using operating-system credential storage when available, with a local fallback if credential storage fails. [VERIFY: production secret-storage behavior and fallback disclosure]

No general telemetry or analytics SDK was identified during repository inspection. [VERIFY: telemetry, analytics, crash-report, and support-data behavior in production builds]

You are responsible for deleting local outputs, downloaded media, generated clips, exported JSON, logs, caches, model files, and project history where the application or operating system allows deletion.

## 14. Licensing, Activation, and Device Binding

The application may require license activation and validation. Licensing may include license keys, device binding, session validation, reset requests, Gumroad purchase verification, purchaser email records, server-side license records, and local license/session/device state.

A license may not work if it is invalid, revoked, expired, already bound to another device, blocked, reset-pending, affected by payment issues, or rejected by the licensing backend.

We are not responsible for access interruptions caused by invalid purchases, chargebacks, Gumroad issues, license server downtime, network failures, device reset delays, device changes, local storage corruption, or unsupported environments.

[VERIFY: whether refund terms, including any 7-day manual refund policy, should be included in this document]

## 15. Third-Party Licenses

Third-party libraries, APIs, services, tools, AI models, datasets, and other components remain governed by their own licenses and terms.

These may include FFmpeg, Python packages, Rust crates, Node/npm packages, Tauri components, Svelte/Vite tooling, OpenCV, yt-dlp, faster-whisper, Whisper-style model files, OpenAI, MuAPI, Gumroad, Cloudflare, and other dependencies or services.

You are responsible for complying with applicable third-party terms where relevant, especially if you redistribute the application, bundle dependencies, distribute model files, use outputs commercially, or process third-party content.

[VERIFY: complete third-party notices, open-source notices, FFmpeg notices, and model license notices before release]

## 16. Prohibited Uses

You must not use the application to:

- Violate any law or regulation.
- Infringe copyright, trademark, privacy, publicity, or other rights.
- Process private, confidential, or sensitive content without permission.
- Harass, abuse, threaten, defame, exploit, or harm others.
- Create or distribute malware or harmful automation.
- Generate spam, deceptive content, or fraudulent content.
- Manipulate platforms, rankings, engagement, recommendations, or monetization systems.
- Scrape, download, or process content without authorization.
- Violate API provider terms or platform terms.
- Misuse API keys, credentials, license keys, or accounts.
- Bypass, tamper with, disable, or interfere with licensing, activation, device binding, security controls, or access controls except where applicable law expressly permits.

## 17. No Warranty

The application is provided "as is" and "as available." To the maximum extent permitted by law, we disclaim all warranties, whether express, implied, statutory, or otherwise, including warranties of merchantability, fitness for a particular purpose, title, non-infringement, accuracy, compatibility, availability, and uninterrupted operation.

We do not guarantee that the application will be error-free, uninterrupted, secure, compatible with your environment, accepted by any platform, monetized, ranked, visible, profitable, or suitable for your intended purpose.

We do not guarantee output correctness, transcript accuracy, highlight quality, title quality, hook quality, media quality, API availability, dependency compatibility, local model performance, FFmpeg success, diagnostics accuracy, or update availability.

## 18. Limitation of Liability

To the maximum extent permitted by law, we are not liable for indirect, incidental, special, consequential, exemplary, punitive, or similar damages.

We are not liable for losses or claims involving data loss, lost revenue, lost profits, account bans, account strikes, API costs, unexpected charges, publishing mistakes, copyright claims, takedowns, rejected uploads, failed uploads, demonetization, corrupted outputs, damaged media files, failed conversions, AI mistakes, local processing errors, dependency failures, third-party service failures, license activation issues, device reset delays, unsupported environments, user-modified settings, user-modified paths, or disclosed diagnostic information.

[VERIFY: whether to include a monetary liability cap, such as fees paid in the previous 12 months]

## 19. Updates and Changes

The application may be updated over time. Updates may add, remove, or change features, APIs, supported providers, dependencies, model options, output formats, diagnostics behavior, settings, licensing behavior, supported operating systems, security behavior, or system requirements.

We do not guarantee that any specific feature, provider, dependency, model, workflow, or output format will remain available.

These Terms may also be updated over time. Continued use of the application after updated Terms become effective means you accept the updated Terms.

## 20. Termination

We may suspend or terminate access to the application or licensing services if we reasonably believe that you violated these Terms, misused a license key, bypassed access controls, reversed payment, used the application unlawfully, harmed third parties, or created legal, security, or operational risk.

You may stop using the application at any time. Termination does not remove your responsibility for content you created, processed, uploaded, published, or distributed while using the application.

## 21. Contact Information

[DEVELOPER NAME]

[CONTACT EMAIL]

[WEBSITE OR SUPPORT URL]

## 22. Governing Law

These Terms are governed by the laws of [JURISDICTION], without regard to conflict-of-law rules.

[VERIFY: governing law, venue, arbitration, class-action waiver, consumer-law requirements, and regional compliance obligations with qualified legal counsel]
