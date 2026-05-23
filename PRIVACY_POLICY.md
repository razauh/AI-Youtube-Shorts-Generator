# Privacy Policy

Last updated: May 23, 2026

This Privacy Policy explains how [APP NAME] handles data based on the currently inspected application behavior. Repository evidence refers to the project as "AI YouTube Shorts Generator" and some application UI references "Signal Forge." [VERIFY: production app name and legal product name]

[APP NAME] is provided by [DEVELOPER NAME]. Contact: [CONTACT EMAIL].

This document should be reviewed by a qualified lawyer before public release.

## 1. Processing Overview

The application supports both local processing and API-based processing.

Local processing may occur on your device when the application processes local files, stores project history, stores settings, checks dependencies, runs local media tools, uses local model files, creates output files, or stores crash drafts and diagnostics.

External processing may occur when you use API mode, configure third-party API keys, activate or validate a license, request a device reset, check for updates, install updates, or submit crash information where a crash-report endpoint is configured.

## 2. Data You Provide or Generate

Depending on your use, the application may handle:

- YouTube URLs or other source references.
- Local video files and downloaded source media.
- Audio, video, transcript text, and generated clips.
- AI prompts or prompt-like processing context.
- Generated titles, hooks, scores, reasons, metadata, and JSON output.
- Output file paths and local project history.
- API keys, API key profile labels, and masked key metadata.
- License keys, masked license information, activation status, reset request state, and device-related license information.
- Logs, crash drafts, diagnostics, error messages, dependency paths, environment details, and configuration values.

## 3. Local Storage

The application may store data locally on your device, including project history, generated output metadata, settings, API key profile metadata, local model profile metadata, model caches, logs, crash drafts, reset status, configuration files, generated media files, exported JSON, and license/session/device-related information.

API key values are intended to be stored using operating-system credential storage when available, with a local fallback if credential storage fails. [VERIFY: production secret-storage behavior and fallback disclosure]

You are responsible for managing and deleting local files, generated outputs, downloaded media, exported JSON, logs, caches, model files, and project history where the application or operating system allows deletion.

## 4. Data Sent to Third Parties

The application may send data to third parties when you choose or configure features that require them.

MuAPI/API mode may receive source URLs, media references, transcript-related data, prompt-like processing data, highlight data, timing data, aspect-ratio settings, and other processing information needed to generate clips.

OpenAI or another configured AI provider may receive transcript text, prompt text, highlight-selection instructions, generated context, and related metadata for AI processing.

License services may receive license activation, validation, reset, device binding, and session information. The licensing backend may store hashed license keys, purchaser email, provider sale identifiers, device binding information, reset request information, audit events, and related metadata.

Gumroad-related verification may be used by the licensing backend to confirm purchase records.

Updater services may receive information needed to check for and install application updates. [VERIFY: production update endpoint and signing configuration]

Crash-report services may receive crash drafts or diagnostic information only if a crash-report endpoint is configured and the user submits a report. [VERIFY: production crash-report endpoint, submission behavior, and retention policy]

## 5. Logs and Diagnostics

Logs, diagnostics, crash drafts, output JSON, and support materials may include sensitive information such as local file paths, dependency paths, usernames embedded in paths, filenames, source URLs, transcript text, prompt text, generated outputs, API errors, environment details, configuration values, processing status, and stack traces.

Review logs and diagnostics before sharing them publicly or with support. We are not responsible if you disclose sensitive information by sharing logs, screenshots, crash drafts, output JSON, or diagnostic details.

## 6. Telemetry, Analytics, and Crash Reports

No general telemetry or analytics SDK was identified during repository inspection.

The application includes crash draft behavior and may support user-submitted crash reports if an endpoint is configured.

[VERIFY: whether production builds collect telemetry, analytics, automatic crash reports, or support diagnostics beyond the inspected behavior]

## 7. API Keys and Credentials

You are responsible for protecting your API keys, credentials, license keys, and accounts.

Do not share API keys, license keys, logs, screenshots, support bundles, or configuration files that expose secrets or sensitive account information.

We are not responsible for unauthorized charges, revoked access, leaked credentials, misconfigured API keys, provider account restrictions, or losses caused by compromised credentials.

## 8. User Content and Rights

You are responsible for ensuring you have permission to process any video, audio, transcript, image, text, likeness, voice, or other content you provide to the application.

The application does not grant you rights to third-party content, music, videos, images, datasets, models, APIs, or platform content.

## 9. Third-Party Services

Third-party services have their own privacy policies, terms, data practices, retention rules, and security practices. Review the policies for MuAPI, OpenAI, Gumroad, Cloudflare, and any other provider you use with the application.

We do not control third-party providers and are not responsible for their data practices.

## 10. Data Retention and Deletion

Local data may remain on your device until you delete it, the application deletes it, or your operating system removes it.

Server-side license, purchase, device binding, reset, audit, support, update, or crash-report data may be retained according to the relevant backend or provider configuration.

[VERIFY: production retention and deletion policy for license records, purchaser email, device binding information, support requests, crash reports, and logs]

## 11. Security

The application uses local and provider-specific mechanisms for storage and processing, including operating-system credential storage where available for secrets. However, no system can be guaranteed secure.

You are responsible for securing your device, operating system account, API keys, license keys, local files, generated outputs, logs, and configuration.

Do not use the application on a device or account you do not trust.

## 12. Children and Sensitive Content

The application is not intended for use by children. [VERIFY: minimum user age and child-directed service policy]

Do not process private, confidential, sensitive, biometric, health, financial, child-related, or legally protected content unless you have all required rights, permissions, consents, and legal basis.

## 13. Changes to This Privacy Policy

This Privacy Policy may be updated over time to reflect changes in the application, services, dependencies, data handling, or legal requirements.

Continued use of the application after changes become effective means you accept the updated policy.

## 14. Contact

[DEVELOPER NAME]

[CONTACT EMAIL]

[WEBSITE OR SUPPORT URL]
