# Terms and Conditions

Last updated: May 25, 2026  
Effective date: [EFFECTIVE DATE]

These Terms and Conditions ("Terms") apply to your installation, activation, access to, and use of [APP NAME]. The repository and desktop bundle identify the product as "AI YouTube Shorts Generator," while parts of the application UI refer to "Signal Forge." [VERIFY: final production product name]

[APP NAME] is provided by [LEGAL COMPANY / DEVELOPER NAME], [COMPANY ADDRESS]. Contact: [SUPPORT EMAIL]. Privacy contact: [PRIVACY EMAIL]. Governing law: [JURISDICTION].

This document is a software-distribution policy draft based on the inspected application codebase. It is not legal advice and must be reviewed by a qualified lawyer before commercial release.

## 1. Acceptance of Terms

By installing, activating, accessing, or using the application, you agree to these Terms. If you do not agree, do not install, activate, or use the application.

If you use the application on behalf of a company or organization, you represent that you are authorized to accept these Terms on its behalf.

The desktop activation flow requires acceptance of the Terms before license activation. Continued use of the application after updated Terms become effective means you accept the updated Terms.

## 2. Description of the Application

The application is a desktop tool for generating short-form video clips from user-provided sources, including YouTube URLs and local video files. The inspected codebase includes:

- A Tauri/Rust desktop backend and Svelte frontend.
- A Python bridge and legacy Python media-processing pipeline.
- API-based generation through MuAPI.
- Local generation using Python, yt-dlp, faster-whisper, FFmpeg, OpenCV, and OpenAI for highlight selection.
- License activation, validation, device binding, device-reset requests, and an admin reset-review console.
- A Cloudflare Worker/D1 licensing backend with Gumroad purchase verification.
- Local settings, project history, diagnostics, crash-report drafts, runtime-pack/model downloads, update checks, and optional output JSON exports.

The application is a tool. It does not guarantee lawful outputs, correct transcripts, good edits, platform acceptance, monetization, engagement, ranking, revenue, business results, or fitness for a specific creator workflow.

## 3. License to Use the Application

Subject to these Terms and any purchase or license terms, [LEGAL COMPANY / DEVELOPER NAME] grants you a limited, non-exclusive, non-transferable, revocable license to install and use the application for lawful personal or internal business purposes.

You may not copy, sublicense, resell, rent, lease, distribute, host, or provide the application as a service unless a separate written agreement permits it.

The application may require a valid license key, session token, and device binding before protected features are available. The main generation UI is intended to be unavailable until the local license state is licensed or within an offline grace state.

## 4. User Responsibilities

You are responsible for your use of the application and for all content, URLs, files, prompts, transcripts, metadata, clips, exports, and outputs that you provide, process, generate, publish, or distribute.

You are responsible for:

- Having all required rights, permissions, licenses, and consents for source media and generated outputs.
- Complying with copyright, trademark, privacy, publicity, platform, API-provider, payment-provider, export-control, and other applicable laws and rules.
- Reviewing transcripts, titles, hooks, scores, reasons, captions, video clips, output JSON, and any AI-generated content before relying on or publishing it.
- Protecting your API keys, license keys, admin tokens, accounts, local files, generated outputs, and device access.
- Backing up files, projects, outputs, settings, and credentials that you need.

The application does not provide legal, copyright, platform-policy, monetization, business, tax, or professional advice.

## 5. Acceptable Use

You may use the application only for lawful purposes and in accordance with these Terms, applicable third-party terms, and applicable law.

You must only process content that you own, control, are licensed to use, or are otherwise legally permitted to process.

## 6. Prohibited Use

You must not use the application to:

- Violate any law, regulation, court order, contract, platform rule, or third-party right.
- Infringe copyright, trademark, privacy, publicity, personality, moral, creator, music, or other rights.
- Download, scrape, extract, process, or redistribute platform content without authorization.
- Process private, confidential, sensitive, biometric, health, financial, child-related, or legally protected content without a valid legal basis and all required consents.
- Generate deceptive, fraudulent, defamatory, harassing, exploitative, abusive, hateful, sexual, violent, illegal, or harmful content.
- Create spam, misleading engagement bait, impersonation, platform manipulation, or content intended to evade moderation systems.
- Upload or distribute malware, harmful automation, credential-harvesting content, or security-abuse content.
- Misuse, share, sell, leak, or bypass API keys, license keys, admin tokens, session tokens, payment records, or device-binding controls.
- Reverse engineer, tamper with, bypass, disable, overload, or interfere with licensing, activation, device binding, security controls, update mechanisms, Worker APIs, or access controls except where applicable law expressly permits.

## 7. User Content and Output Ownership

You retain whatever rights you already hold in your own content. The application does not transfer ownership of your source media to [LEGAL COMPANY / DEVELOPER NAME].

You grant the application and any configured processing providers the limited rights necessary to process your inputs and generate requested outputs. For example, API mode may require sending URLs, hosted media references, transcripts, prompt text, timing data, and crop instructions to MuAPI. Local mode may require sending transcript-derived prompts to OpenAI.

The application does not grant you rights to third-party videos, audio, music, images, faces, voices, likenesses, transcripts, datasets, fonts, models, APIs, software, or platform content.

You are solely responsible for deciding whether generated outputs may be published, monetized, licensed, sold, or otherwise used.

## 8. YouTube and Social-Platform Compliance

The application can process YouTube URLs and local video files, but it is not affiliated with, endorsed by, or certified by YouTube, Google, TikTok, Instagram, Meta, or any other platform unless separately stated in writing.

You are responsible for complying with all platform terms, API terms, copyright policies, downloader restrictions, community guidelines, monetization rules, content-ID systems, takedown processes, and rate limits.

The application does not guarantee that any clip, title, hook, caption, thumbnail, export, or upload will be accepted, ranked, recommended, monetized, or left online by any platform.

## 9. AI-Generated Content Disclaimer

The application may use AI systems for transcription, classification, highlight selection, scoring, title generation, hook generation, virality reasoning, and related text or metadata generation.

AI-generated outputs may be inaccurate, incomplete, biased, offensive, misleading, duplicative, low quality, legally risky, or unsuitable for your intended use.

You must review and verify AI outputs before relying on them or publishing them. [LEGAL COMPANY / DEVELOPER NAME] is not responsible for claims, losses, takedowns, account strikes, demonetization, rejected uploads, or other consequences arising from AI-generated outputs or your use of them.

## 10. Processing Modes

### API Mode

In API mode, the application uses MuAPI for hosted processing. Based on the inspected code, API mode may send MuAPI:

- Source video URLs and requested download format.
- Hosted media URLs returned by MuAPI.
- Language settings for transcription.
- Transcript-derived prompt text and highlight-selection prompts.
- Start/end timestamps, aspect-ratio settings, and autocrop instructions.
- MuAPI API keys in request headers.

MuAPI may perform download, transcription, LLM processing, polling, status reporting, and autocrop/rendering on its own systems.

### Local Mode

In local mode, the application runs a local Python bridge. Local mode can process a local file path directly or use yt-dlp to download a URL to local storage. It can use faster-whisper for local transcription, FFmpeg for cutting/encoding/muxing, OpenCV for face-aware reframing, and OpenAI for highlight-ranking prompts.

Local mode is not necessarily fully offline. It may still contact YouTube or another source platform through yt-dlp, OpenAI for LLM processing, model-hosting services for faster-whisper model downloads, runtime-pack hosts for local runtime downloads, the licensing backend for activation/session validation/reset, update endpoints, and optional crash-report endpoints.

## 11. Third-Party APIs and Services

The application may use or connect to third-party services, including:

- MuAPI for hosted video download, transcription, LLM-style processing, polling, and autocrop/rendering.
- OpenAI for local-mode highlight ranking and prompt-based text generation.
- Gumroad for purchase records and license verification.
- Cloudflare Workers and D1 for the licensing backend.
- YouTube or other source platforms when you provide platform URLs or when yt-dlp accesses those platforms.
- Model-hosting services used by faster-whisper or related tooling when downloading local models.
- Runtime-pack and update hosts configured by the operator.
- A crash-report endpoint only if configured and only when you submit a pending crash draft.

Third-party services are controlled by their providers. We do not control their availability, pricing, retention, security, privacy practices, terms, policies, account decisions, content decisions, rate limits, quotas, or output quality.

You are responsible for complying with all third-party terms and for paying any third-party charges tied to your accounts or API keys.

## 12. FFmpeg, yt-dlp, Python Runtime, and Local Dependencies

The application may use FFmpeg for cutting, encoding, muxing, converting, reframing, and producing media files. It may use yt-dlp for local URL downloading. It may use Python, faster-whisper, OpenCV, OpenAI client libraries, optional CUDA/GPU dependencies, and local or downloaded model files.

These tools and libraries are third-party components governed by their own licenses, terms, policies, and technical constraints. FFmpeg licensing obligations can vary depending on build options, codecs, linking, bundling, and distribution method. yt-dlp and platform access may be restricted by platform terms.

[VERIFY BEFORE RELEASE: whether production builds bundle FFmpeg, yt-dlp, Python, runtime packs, and model files; exact versions; source-offer obligations; GPL/LGPL/commercial codec implications; and model license terms.]

## 13. Local Storage, Files, Logs, and Diagnostics

The application may store local project history, theme preferences, reset status cache, crash-report drafts, runtime context, settings, API-key profile metadata, local-model profile metadata, model caches, runtime-pack files, logs, generated media, output JSON, license/session state, device identity, and fallback secret files on your device.

Logs, diagnostics, crash drafts, output JSON, and support materials may include sensitive or identifying information, including local file paths, usernames embedded in paths, source URLs, transcript text, prompt text, generated outputs, dependency paths, Python runtime details, stack traces, API/provider errors, and processing status.

You should review logs, diagnostics, screenshots, JSON exports, and crash drafts before sharing them. We are not responsible if you disclose sensitive information by sharing those materials.

## 14. API Keys, Credentials, and Admin Tokens

The application allows you to configure MuAPI and OpenAI API-key profiles. The admin desktop app allows authorized operators to configure a Worker base URL and admin API token.

The inspected code attempts to use operating-system credential storage for secrets where available and also includes local fallback storage mechanisms. You are responsible for securing your device account, keychain/credential store, fallback files, API keys, license keys, admin tokens, and configuration files.

We are not responsible for unauthorized third-party API charges, revoked access, leaked credentials, provider account restrictions, or losses caused by compromised or misconfigured credentials.

## 15. Licensing, Activation, Device Binding, and Resets

The application may require a valid license key before generation features are available. The inspected implementation includes:

- License activation and session validation against a license Worker.
- Device binding using a locally generated device public key and device fingerprint information.
- Local license/session state and device identity storage.
- Server-side license records keyed by a hashed license key.
- Session access tokens, masked license-key display, offline grace states, and reauthentication states.
- Device reset requests with pending, approved, rejected, and expired states.
- Manual admin review and approval/rejection of reset requests.

A license may fail or stop working if it is invalid, inactive, expired, revoked, refunded, disputed, already bound to another device, affected by local storage corruption, blocked by network failures, or rejected by the licensing backend.

Device-reset approval may deactivate existing device bindings for the relevant license so that the license can be activated again. Reset approval is not guaranteed and may require admin review.

## 16. Payment Provider and Refunds

Payment processing and purchase records may be handled by Gumroad or another configured payment provider. The application and Worker do not process payment-card numbers in the inspected code; purchase verification uses provider sale data and license information returned by the provider.

Refunds, disputes, chargebacks, taxes, and payment-provider account issues are subject to the payment provider's terms and the final refund policy adopted by [LEGAL COMPANY / DEVELOPER NAME]. [VERIFY: final refund terms, refund window, regional consumer rights, and support process before release.]

Refunded or disputed sales may be ineligible for activation or continued access.

## 17. Updates and Runtime Downloads

The application includes Tauri updater integration and local runtime-pack download/repair flows. Update checks and runtime-pack downloads may contact configured endpoints and may download signed release artifacts, manifests, runtime archives, Python tooling, model dependencies, or related files depending on the final release configuration.

Updates may add, remove, or change features, providers, dependencies, supported platforms, output formats, policies, security behavior, licensing behavior, or system requirements.

We do not guarantee that any specific feature, provider, model, runtime, dependency, output format, or update endpoint will remain available.

## 18. Security Limitations

The application includes security-oriented behavior such as license gating, device binding, masked keys, hashed server-side license keys, safe error mapping, credential storage, and redaction in selected crash drafts and logs. However, no software, local storage mechanism, credential store, network service, or AI system can be guaranteed secure.

You are responsible for securing your device, operating-system user account, local files, generated outputs, logs, keychain, API keys, license keys, admin tokens, and network environment.

Do not use the application on a device or account you do not trust.

## 19. No Warranty

The application is provided "as is" and "as available." To the maximum extent permitted by law, [LEGAL COMPANY / DEVELOPER NAME] disclaims all warranties, whether express, implied, statutory, or otherwise, including warranties of merchantability, fitness for a particular purpose, title, non-infringement, accuracy, compatibility, availability, security, and uninterrupted operation.

We do not warrant that the application, local processing, API processing, licensing services, update services, runtime downloads, model downloads, diagnostics, or third-party integrations will be error-free, uninterrupted, secure, compatible with your environment, or available in any location.

## 20. Limitation of Liability

To the maximum extent permitted by law, [LEGAL COMPANY / DEVELOPER NAME] will not be liable for indirect, incidental, special, consequential, exemplary, punitive, or similar damages.

We are not liable for losses or claims involving data loss, lost revenue, lost profits, account strikes, account bans, takedowns, rejected uploads, failed uploads, demonetization, copyright claims, privacy claims, API charges, unexpected provider bills, corrupted outputs, failed conversions, AI mistakes, inaccurate transcripts, dependency failures, FFmpeg/yt-dlp failures, local model failures, third-party service failures, license activation issues, reset delays, update failures, unsupported environments, user-modified settings, disclosed diagnostics, or use of content without proper rights.

[VERIFY WITH COUNSEL: whether to include a monetary liability cap, consumer-law carveouts, arbitration, class-action waiver, venue clause, and jurisdiction-specific exceptions.]

## 21. Indemnification

To the extent permitted by law, you agree to defend, indemnify, and hold harmless [LEGAL COMPANY / DEVELOPER NAME], its owners, developers, contractors, affiliates, service providers, and licensors from claims, damages, liabilities, losses, costs, and expenses arising from:

- Your use or misuse of the application.
- Your source content, generated outputs, uploads, publications, or distributions.
- Your violation of these Terms, applicable law, third-party rights, platform rules, or provider terms.
- Your use of API keys, payment accounts, license keys, or admin tokens.
- Your publication or monetization decisions.

## 22. Suspension and Termination

We may suspend or terminate access to the application, licensing services, reset workflows, update services, or support if we reasonably believe that you violated these Terms, misused a license key, bypassed access controls, reversed payment, created security risk, harmed third parties, or used the application unlawfully.

You may stop using the application at any time. Termination does not remove your responsibility for content you created, processed, uploaded, published, or distributed while using the application.

## 23. Changes to These Terms

We may update these Terms to reflect changes in the application, providers, licensing, legal requirements, or business practices.

The updated Terms will be effective on the stated effective date. Continued use after that date means you accept the updated Terms.

## 24. Governing Law and Dispute Resolution

These Terms are governed by the laws of [JURISDICTION], without regard to conflict-of-law rules.

Venue, arbitration, class-action waiver, consumer-law exceptions, and regional dispute-resolution requirements are [TO BE COMPLETED AFTER LEGAL REVIEW].

## 25. Contact

[LEGAL COMPANY / DEVELOPER NAME]  
[COMPANY ADDRESS]  
Support: [SUPPORT EMAIL]  
Privacy: [PRIVACY EMAIL]  
Website: [WEBSITE OR SUPPORT URL]

