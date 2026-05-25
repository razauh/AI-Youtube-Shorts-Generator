# Privacy Policy

Last updated: May 25, 2026  
Effective date: [EFFECTIVE DATE]

This Privacy Policy explains how [APP NAME] handles information. The repository and desktop bundle identify the product as "AI YouTube Shorts Generator," while parts of the application UI refer to "Signal Forge." [VERIFY: final production product name]

Operator: [LEGAL COMPANY / DEVELOPER NAME]  
Address: [COMPANY ADDRESS]  
Support contact: [SUPPORT EMAIL]  
Privacy contact: [PRIVACY EMAIL]

This policy is based on the inspected application codebase. It is not legal advice and must be reviewed by a qualified lawyer before commercial release.

## 1. Scope

This policy covers the desktop application, local runtime components, optional API workflows, license activation and validation, the Cloudflare Worker licensing backend, Gumroad purchase verification, the admin reset-review console, diagnostics, local storage, and optional crash-report submission.

It does not replace the privacy policies of third-party services such as MuAPI, OpenAI, Gumroad, Cloudflare, YouTube/Google, model-hosting services, update hosts, or any other provider you configure or use.

## 2. Processing Overview

The application supports both local processing and cloud/API processing.

Local processing may occur on your device when the application reads local files, stores settings and project history, runs the Python bridge, uses FFmpeg/yt-dlp/faster-whisper/OpenCV, creates generated media, writes output JSON, checks dependencies, stores logs, stores crash drafts, downloads runtime packs, or caches local model files.

External processing may occur when you use API mode, configure API keys, activate or validate a license, request or check a device reset, verify Gumroad purchases, check for or install updates, download runtime packs or local models, use yt-dlp with online platforms, use OpenAI in local mode, or submit a crash report to a configured endpoint.

No general telemetry or analytics SDK was identified in the inspected codebase. Crash reports are stored as local drafts and are submitted only if a crash-report endpoint is configured and you choose to submit the draft.

## 3. Information the Application Handles

Depending on your use, the application may handle the following categories of information.

### User Content and Generation Data

- YouTube URLs or other source URLs.
- Local video file paths selected through the desktop file picker.
- Downloaded source media files.
- Audio, video, and image frames processed for clipping, transcription, and reframing.
- Transcript text and segment timestamps.
- Prompt text and transcript-derived prompts used for highlight selection.
- Generated highlight titles, hooks, scores, virality reasons, timing metadata, and clip metadata.
- Generated video files and local or hosted clip URLs.
- Optional detailed output JSON reports.
- Project names, project status, source references, generated short metadata, and library search data stored locally.

### Licensing, Payment, and Device Data

- License keys entered for activation, reauthentication, and device reset.
- Masked license-key values for display.
- Hashed license-key values stored in the licensing backend.
- Purchase email addresses received from Gumroad or reset workflows.
- Gumroad sale IDs, product IDs, refund/dispute eligibility signals, and provider identifiers.
- Entitlement status and provider metadata.
- Device public keys, locally stored device private key material, device IDs, device-binding status, and device fingerprint information.
- Fingerprint fields such as operating system, platform, architecture, app version where available, and a hashed hostname value where collected.
- Access tokens, token expiration times, local session state, offline grace state, reauthentication state, and reset request IDs/status.

### Credentials and Secrets

- MuAPI API keys.
- OpenAI API keys.
- Admin API tokens and admin Worker base URL.
- License keys and access tokens stored locally for licensing.
- Environment variables or `.env` values used for configuration.
- Worker secrets configured by the operator, such as Gumroad access tokens, token-signing secrets, hash peppers, and admin API tokens. These are not intended to be exposed to end users.

### Technical, Diagnostic, and Local Storage Data

- Application version, platform, runtime root, configuration paths, log paths, protected-storage paths, and secure-fallback paths.
- Dependency status for Python, FFmpeg, yt-dlp, faster-whisper, and local runtime packs.
- Runtime-pack status, manifest URL, version, platform, architecture, error codes, and debug references.
- Local model profile labels, selected Whisper model, selected processing device, download status, cache paths, error codes, and debug references.
- Logs, crash-report drafts, error names/messages/stacks, processing status, provider errors, and stack traces.
- Theme preference, reset status cache, and crash drafts stored in desktop web storage equivalents such as `localStorage`.
- File-manager open paths and selected output JSON paths.

## 4. Local Processing and Local Storage

The application stores data on your device. The inspected implementation uses operating-system app-data locations and desktop web storage rather than browser cookies for core app state.

Local storage may include:

- Project/library history in `localStorage`, including project name, source URL or local path, generated short metadata, and clip paths or URLs.
- Theme preference in `localStorage`.
- Device-reset status cache in `localStorage`.
- Crash-report drafts in `localStorage` under a local draft key.
- Runtime configuration, logs, model caches, runtime-pack state, and secure-fallback files under the app's local runtime data directory.
- API-key profile metadata, including profile labels, last-four display values, active profile flags, and timestamps.
- Local model profile metadata, including profile labels, model names, device choices, download states, and timestamps.
- Local generated media and downloaded source media in configured output directories.
- Optional output JSON containing detailed generation results.

The application attempts to use operating-system credential storage for secrets where available. The inspected code also includes local fallback storage for API keys, admin tokens, license keys, access tokens, and device keypair material when keychain/credential storage is unavailable or as part of resilient storage behavior. These fallback files are sensitive and should be protected by the user's operating-system account, disk permissions, and device security.

The frontend includes an AES-GCM encrypted protected-store utility for some protected local payloads, but not all local files or fallback stores should be assumed encrypted. Do not share local configuration, fallback-secret files, logs, screenshots, or support materials unless you have reviewed them.

## 5. Data Sent to MuAPI

When you use API mode, the application may send MuAPI:

- The source video URL and requested format/resolution for `youtube-download`.
- The hosted media URL returned from download processing.
- Language settings and transcription options for `openai-whisper`.
- Transcript text, transcript samples, content-classification prompts, and highlight-selection prompts for `gpt-5-mini` processing through MuAPI.
- Start/end timestamps, aspect ratio, and hosted video URL for `autocrop`.
- A MuAPI API key in the `x-api-key` request header.

MuAPI returns request IDs, polling status, hosted media URLs, transcripts, highlight outputs, rendered clip URLs, errors, and provider payloads. MuAPI's own terms and privacy policy apply.

## 6. Data Sent to OpenAI

In local mode, transcription and clipping may occur locally, but highlight ranking uses OpenAI through the OpenAI Python client if configured. The application may send OpenAI:

- Transcript samples.
- Timestamped transcript text.
- Prompt instructions for content classification, highlight selection, titles, hooks, scores, and virality reasons.
- Related processing context necessary to produce highlight JSON.

The default OpenAI model in the inspected Python configuration is `gpt-4o-mini`, unless overridden by configuration. OpenAI's own terms and privacy policy apply.

## 7. Local Mode Platform and Model Downloads

If local mode is used with a YouTube URL or other online source, yt-dlp may contact YouTube or the relevant platform to download media. That platform may receive network request information according to its own systems and terms.

When local model profiles are created or retried, faster-whisper may download or cache model files through its model-resolution mechanism. The inspected code stores local model caches under an app-specific Hugging Face-style model cache directory. Depending on the selected model and runtime environment, model downloads may contact Hugging Face or another model host.

When the Local Processing Runtime is prepared, the application fetches a runtime-pack manifest and runtime archive from a configured runtime-pack URL. The default inspected manifest URL is a placeholder and must be replaced before production release.

## 8. Licensing Backend Data

The desktop app may send the licensing backend:

- License key during activation or reset request.
- Device public key.
- Device fingerprint information.
- App version.
- Timestamp.
- Access token during session validation.
- Reset request ID during reset-status checks.
- Optional purchaser email if included by a reset workflow or provider flow.

The Cloudflare Worker licensing backend may store in D1:

- Hashed license keys, not raw license keys, for server-side license records.
- Purchaser email addresses.
- Entitlement status.
- Provider name and provider sale ID.
- Device binding records, including device ID, public key, fingerprint JSON, binding status, and update timestamps.
- Reset requests, including request ID, hashed license key where available, masked license key, purchaser email, reset status, and timestamps.
- Idempotency records, including operation name, idempotency key, payload hash, response status, response body, and timestamp. Some response bodies may contain sensitive activation or reset response data such as signed access tokens, masked license keys, device identifiers, or request IDs.
- Audit events, including event type, actor, metadata JSON, and timestamp.

The inspected Worker hashes normalized license keys with SHA-256 and an operator-configured hash pepper before storing license records. It issues signed access tokens containing license-hash and device-binding context. It does not store payment-card details.

## 9. Gumroad and Payment Provider Data

The inspected licensing Worker includes a Gumroad webhook endpoint. It receives Gumroad form data such as sale ID, product ID, and purchaser email, then calls the Gumroad sales API with an operator-configured Gumroad access token to verify the sale.

Gumroad verification may return sale data such as license key, email, sale ID, product ID, refund status, and dispute status. The Worker uses this data to create or update license records and audit events.

Payment-card details and payment processing are handled by Gumroad or the configured payment provider, not by the desktop app or inspected Worker code. Gumroad's own terms and privacy policy apply.

## 10. Admin Console and Support Access

The codebase includes a separate admin desktop console for authorized operators. The admin console stores a Worker base URL and admin API token using the same desktop secure-store/fallback mechanism.

Authorized admins can query the Worker for:

- Overview counts.
- Reset requests.
- License records with license-hash prefixes and masked purchaser email values.
- Device binding records with device IDs, public-key prefixes, license-hash prefixes, masked purchaser email values, and fingerprint summaries.
- Audit event summaries.
- Idempotency record summaries.

The Worker database may contain raw purchaser email values and full internal hashes even when admin API responses return masked or summarized values. Admin access must be limited to authorized personnel with a legitimate support, licensing, fraud-prevention, or security need.

## 11. Logs, Diagnostics, and Crash Reports

The application may create logs and diagnostics locally. Local model download failures may write diagnostic log lines that include model name, device choice, cache path, Python path, bridge path, debug reference, error code, and redacted error text. Advanced diagnostics may display local runtime paths and dependency paths.

Crash drafts are created locally from frontend window errors and unhandled promise rejections. The crash draft includes app version, platform, timestamp, error name, message, and optional stack trace. The crash draft redacts selected license-key and secret patterns, but redaction may not catch every sensitive value.

Crash drafts are not automatically uploaded in the inspected code. If a crash-report endpoint is configured and you choose to submit a pending crash draft, the draft is sent to that endpoint.

## 12. Cookies and Similar Technologies

The inspected desktop app does not use browser cookies for general tracking. It uses desktop web storage and local files for app state, including localStorage entries for projects, theme, reset status cache, and crash drafts.

Third-party websites, APIs, payment providers, update hosts, model hosts, and platforms that you access outside the app or through network calls may use their own cookies or tracking technologies under their own policies.

## 13. Legal Bases Under GDPR

Where GDPR, UK GDPR, or similar laws apply, the operator should identify and document a lawful basis for each processing purpose. Intended lawful bases may include:

- Contract performance: license activation, session validation, device binding, generation workflows requested by the user, update delivery, support, and account/payment-related functionality.
- Legitimate interests: security, fraud prevention, abuse prevention, license enforcement, diagnostics, service reliability, audit logs, admin review, and product support, subject to balancing tests.
- Consent: optional crash-report submission, optional use of user-provided API keys, optional runtime/model downloads where required by law, and optional support submissions.
- Legal obligation: tax, accounting, chargeback, consumer-protection, dispute, and compliance recordkeeping where applicable.

Special-category data should not be intentionally processed unless the user has a valid legal basis and required consents. The app is a general-purpose media tool and cannot determine whether user media contains special-category data.

## 14. US Privacy Rights

Depending on the operator's location, user location, business size, revenue, data volume, and commercial model, US state privacy laws such as the CCPA/CPRA and similar state laws may grant users rights to:

- Know or access categories and specific pieces of personal information.
- Delete personal information, subject to exceptions.
- Correct inaccurate personal information.
- Receive a portable copy of certain information.
- Opt out of certain sales, sharing, targeted advertising, or profiling where applicable.
- Limit certain uses of sensitive personal information where applicable.
- Not be discriminated against for exercising privacy rights.

No advertising, analytics SDK, sale of personal information, or cross-context behavioral advertising workflow was identified in the inspected codebase. The operator must verify this before release and must provide any legally required "Do Not Sell or Share" or similar mechanism if the final business model or integrations require it.

Privacy requests should be sent to [PRIVACY EMAIL]. The operator should verify requester identity before disclosing, deleting, or changing licensing, purchase, or device-binding records.

## 15. Data Retention

Local data remains on the user's device until the user deletes it, the application deletes it, the user clears local app/session data where available, the app is uninstalled, or the operating system removes it.

Server-side license, purchase, device-binding, reset, idempotency, and audit records are retained according to the operator's backend retention configuration. The inspected code does not implement a complete automatic deletion schedule for D1 records.

Recommended release placeholders:

- Local project history: [RETENTION PERIOD OR USER-CONTROLLED].
- Local logs and crash drafts: [RETENTION PERIOD OR USER-CONTROLLED].
- License and purchase records: [RETENTION PERIOD].
- Device bindings and reset requests: [RETENTION PERIOD].
- Audit and idempotency records: [RETENTION PERIOD].
- Crash reports submitted to support: [RETENTION PERIOD].

Do not claim automatic deletion until implemented and verified.

## 16. Deletion, Access, and Correction Requests

Users can locally delete some data by deleting generated files, clearing the local shorts library, dismissing crash drafts, deleting API-key profiles, deleting local model profiles, clearing sessions, uninstalling the app, or deleting app-data files through the operating system.

Backend deletion, access, or correction requests for license records, purchaser email, provider sale IDs, device bindings, reset requests, audit events, and support/crash records require an operator-side process. The inspected code does not provide a complete in-app self-service backend deletion workflow.

Requests should be sent to [PRIVACY EMAIL]. Some records may need to be retained for fraud prevention, chargebacks, tax/accounting, security, legal claims, licensing enforcement, or compliance obligations.

## 17. International Transfers

The application and its third-party providers may process data in countries other than the user's country. This may include the United States, the European Economic Area, the United Kingdom, or other countries where the operator, Cloudflare, Gumroad, MuAPI, OpenAI, model hosts, update hosts, or support providers operate.

Where GDPR, UK GDPR, or similar transfer rules apply, the operator should implement appropriate transfer safeguards, such as data processing agreements, Standard Contractual Clauses, UK Addenda, transfer impact assessments, or other lawful transfer mechanisms as needed.

## 18. Security Measures and Limitations

The inspected implementation includes several security-oriented measures:

- License-gated access to generation features.
- Server-side hashed license-key records using a hash pepper.
- Signed access tokens for license sessions.
- Device binding using device public keys and fingerprint information.
- Masked license-key and masked purchaser-email display in user/admin views where implemented.
- Safe auth error messages in the frontend.
- Selected redaction for crash drafts and debug output.
- Operating-system credential storage where available.
- Local encrypted protected-store utility for some protected payloads.
- Runtime-pack checksum validation before installation.

Important limitations:

- Local fallback secret files may contain sensitive secrets and should not be assumed encrypted.
- Logs and diagnostics may include local paths, source URLs, prompts, transcripts, and environment details.
- Redaction is best-effort and may not catch every secret.
- Backend retention/deletion automation is incomplete in the inspected code.
- Security depends on final production configuration, including HTTPS, Worker secrets, admin token handling, update signing, runtime-pack hosting, Tauri security settings, and access controls.
- No system can be guaranteed secure.

## 19. Children

The application is not intended for children. [VERIFY: minimum age, child-directed-service status, and parental-consent requirements before release.]

Do not use the application to process child-related personal information or content unless you have all required rights, consents, and legal bases.

## 20. Third-Party Processors and Services

The application may involve the following third-party services depending on configuration and user choices:

- MuAPI for API-mode hosted video processing.
- OpenAI for local-mode LLM prompt processing.
- Gumroad for purchase/license records and payment-provider workflows.
- Cloudflare Workers and D1 for the licensing backend.
- YouTube/Google or other platforms when source URLs are accessed or downloaded.
- Model-hosting providers such as Hugging Face when local models are downloaded through faster-whisper tooling.
- Runtime-pack and update hosts configured by the operator.
- Crash-report endpoint provider if configured and used.
- Operating-system credential stores and local filesystem services.

Each third party has its own terms, privacy policy, security practices, and retention rules.

## 21. Changes to This Policy

This Privacy Policy may be updated to reflect changes in the application, providers, data handling, licensing, support practices, or legal requirements.

The updated policy will be effective on the stated effective date. Continued use after that date means you accept the updated policy where permitted by law.

## 22. Contact

[LEGAL COMPANY / DEVELOPER NAME]  
[COMPANY ADDRESS]  
Support: [SUPPORT EMAIL]  
Privacy requests: [PRIVACY EMAIL]  
Website: [WEBSITE OR SUPPORT URL]

