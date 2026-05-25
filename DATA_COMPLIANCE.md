# Data Compliance

Last updated: May 25, 2026  
Effective date: [EFFECTIVE DATE]

Product: [APP NAME] / AI YouTube Shorts Generator / Signal Forge [VERIFY FINAL PRODUCT NAME]  
Operator: [LEGAL COMPANY / DEVELOPER NAME]  
Address: [COMPANY ADDRESS]  
Privacy contact: [PRIVACY EMAIL]  
Support contact: [SUPPORT EMAIL]

This document is a practical compliance guide based on the inspected application codebase and is intended to support privacy, security, and legal review before release. It is not legal advice. Exact obligations depend on the operator's location, user locations, business size, revenue, data volume, payment model, processor contracts, and final production configuration. A qualified lawyer should review this document before commercial release.

## 1. Purpose and Scope

This document describes data categories, data flows, privacy expectations, security controls, and release-readiness items for the desktop application, local runtime, API processing, licensing backend, payment-provider verification, admin console, diagnostics, and optional crash-report workflow.

It focuses on United States and European Union privacy/data-protection expectations, with notes for UK GDPR where appropriate.

## 2. System Overview

The inspected application includes:

- A Tauri/Rust desktop app with Svelte UI.
- A Python bridge and Python media-processing pipeline.
- API mode using MuAPI for hosted video processing.
- Local mode using yt-dlp, faster-whisper, OpenAI, FFmpeg, and OpenCV.
- Local runtime-pack and local model download flows.
- License activation, validation, session state, device binding, and reset requests.
- A Cloudflare Worker/D1 licensing backend.
- Gumroad purchase verification.
- A separate admin desktop console for reset review and license/device/audit visibility.
- Local project history, diagnostics, logs, crash-report drafts, and optional output JSON.

## 3. Data Inventory

| Category | Examples | Location | Notes |
| --- | --- | --- | --- |
| Source references | YouTube URLs, local video paths | Local UI/state, MuAPI in API mode, yt-dlp/platforms in local URL mode | URLs and paths may identify users, creators, projects, or private files. |
| Source media | Downloaded videos, local videos, audio/video frames | Local disk in local mode; MuAPI in API mode | Treat as user content. May contain personal or sensitive data. |
| Transcripts | Text segments, timestamps, duration | Local memory/output JSON; MuAPI/OpenAI depending mode | Transcript text can contain personal data or sensitive data. |
| Prompts | Content classification and highlight prompts | MuAPI in API mode; OpenAI in local mode | Prompt text includes transcript excerpts and generation instructions. |
| Generated outputs | Clips, titles, hooks, scores, virality reasons, clip URLs/paths | Local project library, local output files, output JSON, MuAPI hosted URLs | User must review before publication. |
| Project history | Project names, sourceUrl, short metadata, updatedAt | LocalStorage | Can include URLs or local paths. |
| API credentials | MuAPI keys, OpenAI keys | OS credential store and/or local fallback | Fallback storage is sensitive. |
| Admin credentials | Worker base URL, admin API token | OS credential store and/or local fallback | Admin token grants backend visibility/actions. |
| License credentials | License key, access token | OS credential store and local fallback in inspected code | Sensitive. Not intended for logs or display. |
| License records | Hashed license key, entitlement, provider, sale ID | Cloudflare D1 | Server stores hashes, not raw license keys, for license records. |
| Purchaser data | Purchaser email, Gumroad sale/product IDs | Gumroad, Worker/D1 | Email is personal data. Admin API masks in several responses. |
| Device data | Device public key, local private key material, device ID, fingerprint JSON | Local app data; Worker/D1 | Device fingerprint includes OS/platform/arch and hashed hostname where collected. |
| Reset data | Reset request ID, status, masked license, purchaser email, timestamps | Local reset cache; Worker/D1; admin console | Manual admin review flow. |
| Audit/idempotency | Event type, actor, metadata JSON, payload hash, response body | Worker/D1 | Response bodies may include sensitive activation/reset response data; retention policy not fully implemented in code. |
| Logs/diagnostics | Runtime paths, dependency status, model errors, debug refs, stack traces | Local logs/localStorage; optional crash endpoint | May include local usernames in paths and source details. |
| Runtime/model data | Runtime-pack status, model profile, cache paths, model files | Local app data; runtime/model hosts | Downloaded files may have separate licenses. |

## 4. Personal Data Categories

The application may process personal data, including:

- Names or identifiers embedded in local file paths, project names, media filenames, URLs, transcripts, or generated content.
- Purchaser email addresses received from Gumroad or reset workflows.
- Provider sale IDs and product IDs tied to purchases.
- License status and entitlement records associated with a purchaser.
- Device identifiers, public keys, device binding records, app version, OS/platform/architecture, and hashed hostname values.
- Local IP/network metadata visible to third-party APIs, update hosts, runtime-pack hosts, model hosts, Gumroad, Cloudflare, or source platforms through normal network requests.
- Admin/support actions tied to reset decisions and audit events.
- Crash-report draft contents if submitted.

## 5. Sensitive or Security-Relevant Data

The application may handle sensitive or security-relevant data even when it is not legally classified as sensitive personal information:

- License keys.
- Access tokens.
- MuAPI API keys.
- OpenAI API keys.
- Admin API tokens.
- Worker secrets configured by the operator.
- Device private key material stored locally.
- Local fallback secret files.
- User media that may contain faces, voices, children, biometrics, health information, financial information, or other sensitive content.
- Transcripts and prompts that may reveal private or confidential information.
- Logs and diagnostics containing paths, runtime details, source URLs, and stack traces.

Policy requirement: do not request or encourage users to process sensitive or special-category content unless the operator has a documented legal basis, consent process, security model, and support workflow.

## 6. Data Flow Overview

### Activation and License Validation

1. User enters a license key after accepting Terms.
2. Desktop app sends license key, device public key, device fingerprint, app version, and timestamp to the license Worker.
3. Worker normalizes and hashes the license key with an operator-configured hash pepper.
4. Worker checks D1 license records and writes/updates device binding records.
5. Worker returns a signed access token, masked license key, entitlement, bound device details, and token expiration.
6. Desktop stores local license/session/device state using OS credential storage and local fallback mechanisms.
7. Later session validation sends the access token to the Worker.

### Gumroad Purchase Verification

1. Gumroad sends a webhook form payload containing sale ID, product ID, and purchaser email.
2. Worker calls Gumroad's sales API with an operator-configured access token.
3. Worker verifies sale ID, product ID, email, refund status, dispute status, and license key availability.
4. Worker hashes the verified license key and stores license entitlement, purchaser email, provider, provider sale ID, and audit metadata in D1.

### Device Reset Workflow

1. User requests a device reset with a license key from the activation screen or settings.
2. Desktop sends license context, device public key, fingerprint, app version, and timestamp to the Worker.
3. Worker stores a reset request with hashed license key where available, masked license key, optional purchaser email, status, and timestamps.
4. User or app checks reset status by request ID.
5. Admin console lists reset requests and can approve or reject pending requests with an admin token.
6. On approval, Worker deactivates active device bindings for the license hash and records audit/idempotency data.

### API Generation Mode

1. User provides a URL and generation settings.
2. Desktop sends source URL and settings to MuAPI.
3. MuAPI downloads/hosts media and returns hosted URL(s).
4. Desktop sends hosted media URL to MuAPI transcription.
5. Desktop sends transcript-derived prompts to MuAPI LLM endpoint.
6. Desktop sends crop timestamps and aspect ratio to MuAPI autocrop.
7. Desktop receives transcripts, highlights, clip URLs, and provider payloads.
8. User may store project history locally and optionally export full output JSON.

### Local Generation Mode

1. User provides a local file path or URL.
2. If a local file path is used, the app processes the local file. If a URL is used, yt-dlp downloads media from the source platform.
3. faster-whisper transcribes audio locally using local model files.
4. OpenAI receives transcript-derived prompts for highlight selection.
5. FFmpeg and OpenCV create local clips and reframe outputs.
6. Generated media and metadata are stored locally and optionally exported to JSON.

### Diagnostics and Crash Drafts

1. App checks runtime/dependency status and displays local paths and status.
2. Local model download failures may write diagnostic logs.
3. Frontend errors create local crash drafts with selected redaction.
4. Crash drafts are submitted only if a crash endpoint is configured and the user chooses to submit.

## 7. Local Processing vs Cloud/API Processing

Local processing claims must be precise. The inspected local mode performs transcription and media rendering locally, but local mode may still use network services for:

- OpenAI prompt processing.
- yt-dlp downloads from YouTube or other platforms.
- Runtime-pack downloads.
- faster-whisper/model downloads.
- License activation, validation, and reset.
- Update checks and installs.
- Optional crash-report submission.

Do not market the application as fully offline unless a separate build disables or replaces all network-dependent workflows and this is verified.

## 8. Licensing Backend Data Controls

Designed safeguards identified in code:

- Server-side license records use hashed license keys rather than raw license keys.
- License hashes use an operator-configured hash pepper.
- Access tokens are signed with an operator-configured token-signing secret.
- Admin API requires a bearer admin token.
- Admin endpoints return masked purchaser emails, hash prefixes, public-key prefixes, and summaries for several views.
- Audit events and idempotency records support review and replay handling.

Open items:

- Define backend retention periods.
- Define data deletion/anonymization process.
- Verify admin token rotation, storage, logging, and least-privilege controls.
- Verify rate limits and abuse protection for public Worker routes.
- Verify D1 backup/export/deletion procedures.
- Verify production HTTPS-only configuration.

## 9. Admin App Data Access

The admin desktop console can access licensing operational data through authenticated Worker APIs. Admin access includes operational counts, reset requests, masked license/purchaser data, device binding summaries, audit summaries, and idempotency summaries.

Compliance requirements:

- Restrict admin tokens to authorized personnel only.
- Use role-based access or separate tokens if multiple admin roles are introduced.
- Log admin decisions without storing unnecessary free-form personal data.
- Train admins not to copy purchaser emails, license hashes, device IDs, or reset data into unapproved systems.
- Provide a documented reset approval/rejection policy.
- Rotate admin tokens after personnel changes or suspected exposure.

## 10. API Keys and User-Provided Credentials

Users may provide MuAPI and OpenAI API keys. Admins may provide Worker admin tokens.

Expected handling:

- Use password fields in UI.
- Store profile metadata separately from secret values.
- Display only last-four or redacted tokens where possible.
- Avoid putting full keys in localStorage, logs, errors, screenshots, tests, or support bundles.
- Allow users to delete API-key profiles.

Risk note: inspected local fallback secret mechanisms can store secrets in local files. Release documentation should clearly tell users to secure their device account and avoid sharing app-data folders.

## 11. Logs and Diagnostics

Logs and diagnostics should be treated as potentially sensitive because they may include:

- Full local paths and usernames.
- Source URLs.
- Model/cache paths.
- Python and bridge paths.
- Runtime status and platform details.
- Error messages and stack traces.
- Transcript/prompt/provider fragments in some failures.

Compliance expectations:

- Do not upload logs automatically without opt-in.
- Redact license keys, API keys, admin tokens, access tokens, raw emails, and machine identifiers where practical.
- Provide user-visible review before support submission where possible.
- Document retention and deletion for submitted crash/support records.

## 12. GDPR Considerations

The operator may be a controller for licensing records, payment-provider metadata, support/admin decisions, crash reports, and user relationship data. Depending on configuration, the operator may also be a controller or processor for user media and generated content processed through hosted services.

The operator should complete:

- Records of Processing Activities where required.
- Data Processing Agreements with processors/subprocessors.
- Transfer Impact Assessments for international transfers where required.
- Legitimate Interests Assessments for fraud prevention, license enforcement, security logging, and diagnostics.
- Data Protection Impact Assessment if processing is likely high risk, such as large-scale sensitive media, biometric/face/voice data, children's data, or systematic monitoring.

## 13. GDPR Legal Bases

Potential lawful bases to document:

| Purpose | Potential lawful basis | Notes |
| --- | --- | --- |
| License activation and validation | Contract performance | Required to provide paid app access. |
| Device binding and reset workflow | Contract performance; legitimate interests | Supports license enforcement and fraud prevention. |
| Gumroad sale verification | Contract performance; legal obligation; legitimate interests | Also supports refunds/disputes/tax/accounting where applicable. |
| Generation requested by user | Contract performance or user instruction | Hosted processing requires third-party disclosure. |
| API-key storage | Contract performance; user instruction | User chooses to configure providers. |
| Crash-report submission | Consent or legitimate interests | Prefer explicit user action/consent. |
| Logs and security audit events | Legitimate interests | Complete balancing test. |
| Legal/tax/chargeback records | Legal obligation; legitimate interests | Retention depends on jurisdiction. |
| Marketing, if added later | Consent or legitimate interests depending region | No marketing/analytics workflow identified in inspected code. |

Do not claim GDPR compliance as guaranteed. State that the application is designed to support compliance when operated with appropriate contracts, settings, retention schedules, and request-handling procedures.

## 14. GDPR Data Subject Rights

Where GDPR applies, users may have rights to:

- Access personal data.
- Receive information about processing.
- Correct inaccurate data.
- Delete data, subject to exceptions.
- Restrict processing.
- Object to processing based on legitimate interests.
- Receive portable data in certain cases.
- Withdraw consent where processing is based on consent.
- Lodge a complaint with a supervisory authority.

Operational requirements:

- Provide a privacy request channel: [PRIVACY EMAIL].
- Verify identity before acting on license/payment/device records.
- Define response timelines and escalation owners.
- Identify records that can be deleted, anonymized, corrected, or must be retained.
- Document exceptions for fraud prevention, payment disputes, tax/accounting, security, legal claims, and licensing enforcement.

## 15. ePrivacy and Terminal-Equipment Considerations

The desktop app stores data on the user's device using local files, OS credential stores, and desktop web storage. In the EU/EEA, ePrivacy-style rules may apply to storing or accessing information on a user's terminal equipment.

Storage necessary for requested app functionality, licensing, security, settings, and local project history may be treated differently from nonessential analytics or advertising storage. No advertising or analytics SDK was identified. If analytics, telemetry, marketing pixels, or nonessential tracking are added later, obtain appropriate consent and update policies.

## 16. UK GDPR Note

For UK users, UK GDPR and the Data Protection Act 2018 may apply. The operator should assess:

- UK representative requirements if the operator is outside the UK.
- UK International Data Transfer Agreement or UK Addendum for transfers.
- ICO registration and fee requirements.
- UK-specific privacy notice language and consumer-law requirements.

## 17. CCPA/CPRA-Style US Privacy Considerations

If the operator is subject to the CCPA/CPRA or similar US state privacy laws, the operator should provide rights to:

- Know/access categories and specific pieces of personal information.
- Delete personal information, subject to exceptions.
- Correct inaccurate personal information.
- Opt out of sale/share or targeted advertising where applicable.
- Limit use/disclosure of sensitive personal information where applicable.
- Appeal denied requests where required by state law.
- Avoid discriminatory treatment for rights exercise.

No sale, sharing for cross-context behavioral advertising, or targeted advertising workflow was identified in the inspected code. The operator must verify this for final production, especially if analytics, ads, affiliate tracking, referral tracking, or marketing integrations are added.

## 18. General US State Privacy Considerations

Other US state privacy laws may require:

- Clear notices at or before collection.
- Data minimization and purpose limitation.
- Consumer rights request workflows.
- Processor contracts.
- Reasonable security safeguards.
- Sensitive data consent or opt-out controls.
- Data protection assessments for high-risk processing.

Applicability depends on thresholds such as revenue, number of consumers, percentage of revenue from data sale, and data categories processed.

## 19. Data Minimization

Recommended minimization controls:

- Store only hashed license keys on the backend.
- Avoid storing raw purchaser emails in audit metadata unless necessary.
- Avoid storing full fingerprints where summaries or hashes are enough.
- Avoid including transcripts, prompts, or source URLs in logs unless necessary for support.
- Keep local project history limited to what the library UI requires.
- Make output JSON optional and user-selected.
- Do not upload crash drafts automatically.
- Redact local paths and usernames in support exports where possible.
- Avoid retaining idempotency response bodies longer than necessary because activation replay records may contain signed access tokens or device-binding response data.

## 20. Purpose Limitation

Data collected for one purpose should not be reused for unrelated purposes without a valid legal basis and updated notice.

Examples:

- Purchaser email should be used for purchase, support, license, reset, refund, fraud-prevention, and legal/accounting purposes, not unrelated marketing unless separately permitted.
- Device binding data should be used for license enforcement, security, reset workflow, and support, not unrelated tracking.
- User media/transcripts/prompts should be used only to provide the requested generation workflow and support chosen by the user.
- Crash reports should be used for debugging and support, not profiling or marketing.

## 21. Retention Principles

The inspected code does not implement a complete backend retention schedule. Before release, define and document retention periods.

Recommended retention approach:

- Local files: user-controlled; provide clear deletion instructions.
- Crash drafts: local until dismissed/submitted; submitted reports retained for [RETENTION PERIOD].
- License records: retain while license is active and for [RETENTION PERIOD] after expiration/refund/termination as needed for legal, fraud, and accounting purposes.
- Device bindings: retain while needed for license enforcement and reset history; delete or anonymize after [RETENTION PERIOD].
- Reset requests: retain while pending and for [RETENTION PERIOD] after decision for audit/fraud purposes.
- Audit events: retain for [RETENTION PERIOD] consistent with security and legal requirements.
- Idempotency records: retain short term unless needed for disputes; consider [RETENTION PERIOD].
- Gumroad/payment metadata: retain according to tax/accounting and payment-dispute requirements.

## 22. User Access, Deletion, and Correction Requests

The operator should implement a documented workflow for:

- Receiving requests at [PRIVACY EMAIL].
- Verifying identity using license key, purchase email, sale ID, or other safe evidence without requesting excessive data.
- Exporting relevant backend records in a human-readable form.
- Correcting purchaser email or entitlement metadata where appropriate.
- Deleting or anonymizing eligible records.
- Preserving records required for chargebacks, fraud prevention, tax, legal claims, or security.
- Recording request outcome and date.

The inspected desktop app provides local controls for some local data, but backend privacy rights require operator-side processes.

## 23. Security Measures

Security controls identified or expected:

- License-gated UI for generation features.
- Device binding and signed access tokens.
- Server-side hashed license keys with hash pepper.
- Admin bearer-token authentication.
- Masked emails/license keys in several UI/API responses.
- Local secure-store use where available.
- Crash-draft redaction for selected secret/license patterns.
- Runtime-pack checksum validation.
- Structured error mapping to avoid exposing raw auth failures.
- Avoidance of automatic telemetry/analytics in inspected code.

Additional release controls recommended:

- Enforce HTTPS for hosted Worker/update/runtime endpoints.
- Set and rotate strong Worker secrets.
- Add rate limiting and abuse controls to public Worker routes.
- Implement admin token rotation and least privilege.
- Review Tauri Content Security Policy and updater signing before production.
- Encrypt or eliminate plaintext local fallback secrets where feasible.
- Add server-side retention/deletion tooling.
- Run dependency/license/security audits before each release.

## 24. Breach Response Considerations

Before release, the operator should create an incident response plan covering:

- Suspected exposure of Worker secrets, Gumroad tokens, admin tokens, API keys, license keys, access tokens, or D1 data.
- Compromised runtime-pack/update hosting.
- Malicious or corrupted runtime/model downloads.
- Unauthorized admin access.
- Accidental support log or crash-report disclosure.
- User notification and regulatory notification timelines under applicable law.
- Token revocation, secret rotation, forensic preservation, and post-incident remediation.

## 25. International Transfer Considerations

Data may move across borders through Cloudflare, Gumroad, MuAPI, OpenAI, update hosts, runtime-pack hosts, model hosts, crash-report endpoints, and support operations.

For EU/EEA or UK data, assess:

- Processor locations.
- Subprocessor lists.
- Standard Contractual Clauses or UK transfer mechanisms.
- Transfer Impact Assessments.
- Data localization or regional processing options.
- Whether user media/transcripts are sent to third countries.

## 26. Processor and Subprocessor Table

| Provider / recipient | Role to assess | Data involved | Purpose | User choice / trigger | Release status |
| --- | --- | --- | --- | --- | --- |
| Cloudflare Workers/D1 | Processor or infrastructure provider | License hashes, emails, device bindings, reset data, audit/idempotency records | Licensing backend | Activation, validation, reset, admin | DPA and retention review required. |
| Gumroad | Independent controller or processor, depending terms | Sale ID, product ID, purchaser email, license key, refund/dispute signals | Purchase and license verification | Purchase/webhook | Terms/DPA/refund review required. |
| MuAPI | Processor or independent provider, depending terms | URLs, hosted media references, transcripts, prompts, crop settings, generated clip URLs | API-mode generation | User selects API mode | Terms/DPA/content retention review required. |
| OpenAI | Processor or provider under API terms | Transcript-derived prompts and processing context | Local-mode highlight ranking | User configures OpenAI/local mode | API terms/data-use review required. |
| YouTube/Google and other platforms | Independent controllers/platforms | URL requests, media access, network metadata | Source media access/download | User provides URL or yt-dlp accesses platform | Platform terms compliance required. |
| Model host such as Hugging Face | Processor/independent provider depending use | Model download request metadata, model files | Local model download/cache | User creates/retries model profile | Model license and host terms review required. |
| Runtime-pack host | Processor/infrastructure provider | Platform/arch request metadata, runtime archive downloads | Local runtime setup | User downloads runtime pack | Host, signing, retention review required. |
| Update host | Processor/infrastructure provider | App version, platform, arch, update request metadata | Update checks/installs | User checks/installs updates | Endpoint/signing review required. |
| Crash-report endpoint | Processor/support provider | Crash draft, app version, platform, stack/error | User-submitted diagnostics | User submits draft | Provider/retention review required. |
| Admin operators/support staff | Internal recipients | Reset, license, device, audit summaries; backend records if directly accessed | Support, fraud prevention, reset review | Admin action | Access policy and training required. |

## 27. Sensitive-Data Handling

The app should not be positioned as designed for regulated sensitive data. Users should be instructed not to process private, confidential, health, biometric, financial, child-related, or special-category content unless they have all required rights and legal bases.

If the operator decides to support sensitive-data use cases, complete a separate DPIA/security review, strengthen encryption and retention controls, define support restrictions, add processor terms, and update all policies.

## 28. Children's Data

The app is not intended for children. Before release, define the minimum age and child-directed-service position.

If users may process videos containing children, the operator should disclose that user media may contain children's personal data and require users to obtain all required rights and consents. Do not knowingly collect children's data for licensing/support accounts without a compliant parental-consent process if required.

## 29. AI-Generated Content and User Responsibility

AI outputs can be incorrect, biased, offensive, misleading, infringing, or unsuitable for publication. The user remains responsible for reviewing outputs and ensuring they have rights to source media and generated content.

Compliance implications:

- Do not claim AI outputs are legally cleared.
- Do not claim platform monetization or acceptance is guaranteed.
- Disclose cloud AI processing where applicable.
- Provide user responsibility language for copyright, publicity rights, privacy rights, and platform rules.

## 30. Compliance Checklist Before Release

Complete these items before commercial release:

- Fill in company name, address, privacy email, support email, website, governing law, and effective dates.
- Have qualified counsel review Terms, Privacy Policy, Third-Party Notices, Data Compliance, refund policy, consumer terms, and regional notices.
- Sync in-app policy content with the final markdown policy documents if the app displays policy text internally.
- Verify final production product name across UI, bundle metadata, policies, purchase pages, and support materials.
- Generate npm, Cargo, and Python dependency license reports.
- Verify FFmpeg, yt-dlp, Python runtime, runtime-pack, and model redistribution obligations.
- Verify model licenses for all selectable faster-whisper/Whisper models.
- Confirm MuAPI, OpenAI, Gumroad, Cloudflare, update host, runtime-pack host, model host, and crash-report provider terms and DPAs.
- Define backend retention periods and implement deletion/anonymization tooling.
- Document privacy request intake, identity verification, fulfillment, denial, and appeal process.
- Confirm no analytics/telemetry/remote logging has been added without policy updates and consent where required.
- Review Worker authentication, secrets, admin token rotation, rate limits, abuse controls, and audit logging.
- Review Tauri CSP, permissions/capabilities, updater signing/public key, and runtime-pack verification.
- Verify crash-report redaction and user-submitted-only behavior.
- Document support access rules and admin reset decision policy.
- Create incident response and breach-notification playbooks.
- Confirm payment/refund/chargeback policy with Gumroad and consumer-law counsel.

## 31. Open Legal and Compliance Items

The following items require legal or operator decisions:

- Final legal entity and contact information.
- Final governing law, venue, arbitration, class-action waiver, and consumer-law carveouts.
- Final refund policy and regional consumer cancellation rights.
- Data Processing Agreements and subprocessor disclosures.
- Retention periods for all backend and support records.
- Whether the operator is subject to GDPR, UK GDPR, CCPA/CPRA, other US state privacy laws, or other regional laws.
- Whether a DPO, EU representative, UK representative, or privacy registration is required.
- Whether model/output disclosures must be added for specific jurisdictions or platforms.
- Whether additional consent is needed for crash reports, local storage, runtime downloads, or sensitive-content workflows.
- Whether server-side deletion and export tooling must be implemented before release.

## 32. Legal-Review Disclaimer

This document is an engineering and documentation aid. It is not legal advice and does not guarantee compliance with GDPR, UK GDPR, CCPA/CPRA, US state privacy laws, consumer-protection laws, payment-provider rules, platform policies, copyright law, AI regulations, or open-source license obligations.

The operator must obtain review by qualified legal counsel before release and whenever the application, providers, data flows, licensing, telemetry, support practices, or business model materially change.

