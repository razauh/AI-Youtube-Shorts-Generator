export type PolicyTab = 'terms' | 'privacy' | 'compliance' | 'notices' | 'refund';

type PolicySection = {
  heading: string;
  paragraphs: string[];
};

export const POLICY_SECTIONS: Record<PolicyTab, PolicySection[]> = {
  "terms": [
    {
      "heading": "Terms and Conditions",
      "paragraphs": [
        "Last updated: May 23, 2026",
        "These Terms and Conditions (\"Terms\") apply to your use of [APP NAME]. Repository evidence refers to the project as \"AI YouTube Shorts Generator\" and some application UI references \"Signal Forge.\" [VERIFY: production app name and legal product name]",
        "[APP NAME] is provided by [DEVELOPER NAME]. Contact: [CONTACT EMAIL]. Governing law: [JURISDICTION].",
        "This document is a practical policy draft for the application and should be reviewed by a qualified lawyer before public release."
      ]
    },
    {
      "heading": "1. Acceptance of Terms",
      "paragraphs": [
        "By installing, activating, accessing, or using the application, you agree to these Terms. If you do not agree, do not use the application.",
        "If you use the application on behalf of a company or organization, you represent that you have authority to accept these Terms on its behalf."
      ]
    },
    {
      "heading": "2. Description of the Application",
      "paragraphs": [
        "The application is a desktop tool for generating short-form video clips from user-provided sources such as YouTube URLs and local video files. Depending on mode and configuration, the application may download or process media, transcribe audio, identify highlight candidates, generate titles/hooks/scores/reasons, create vertical clips, export JSON, store local project history, run dependency checks, and use license activation or validation services.",
        "The application is a tool. It does not guarantee perfect results, lawful outputs, platform acceptance, monetization, engagement, ranking, visibility, or business results.",
        "You are responsible for reviewing and verifying all generated outputs before publishing, sharing, uploading, selling, relying on, or distributing them."
      ]
    },
    {
      "heading": "3. User Responsibilities",
      "paragraphs": [
        "You are responsible for your use of the application and for all content you upload, import, process, generate, export, publish, or distribute.",
        "You are responsible for:",
        "- Ensuring you have rights and permissions for source media and generated outputs.",
        "- Complying with copyright, licensing, privacy, publicity rights, platform rules, API provider terms, and applicable law.",
        "- Reviewing generated video, transcripts, titles, hooks, metadata, scores, and JSON output before use.",
        "- Maintaining valid API keys, credentials, license keys, accounts, dependencies, local files, and configurations.",
        "- Backing up your source files, generated files, projects, configuration, and outputs.",
        "The application does not provide legal, copyright, platform-policy, business, or professional advice."
      ]
    },
    {
      "heading": "4. Third-Party APIs and Services",
      "paragraphs": [
        "The application may use or connect to third-party APIs and services, including MuAPI, OpenAI, Gumroad, Cloudflare Worker services, update services, crash-report endpoints if configured, and any other provider you connect to the application.",
        "Third-party APIs and services are controlled by their respective providers. We do not control their availability, pricing, rate limits, account policies, content policies, output quality, security practices, privacy practices, or terms.",
        "We are not responsible for:",
        "- API downtime, interruptions, latency, or outages.",
        "- Pricing changes, billing disputes, unexpected API costs, or usage charges.",
        "- Rate limits, quota exhaustion, account suspensions, account bans, or provider restrictions.",
        "- Provider policy changes or rejected requests.",
        "- Incorrect, incomplete, unsafe, or delayed API responses.",
        "- Expired, revoked, leaked, restricted, invalid, or misconfigured API keys.",
        "- Losses caused by third-party services or credentials.",
        "You are responsible for complying with all provider terms, policies, usage limits, billing requirements, and content rules."
      ]
    },
    {
      "heading": "5. API Mode and MuAPI Processing",
      "paragraphs": [
        "In API mode, the application may send processing inputs to MuAPI. These inputs may include source URLs, media references, transcript-related data, highlight data, prompt-like processing data, timing data, aspect-ratio settings, and other information needed to generate clips.",
        "MuAPI may perform download, transcription, highlight processing, LLM-based ranking, autocrop, and media-rendering tasks. MuAPI is a third-party service, and its own terms and policies apply.",
        "We are not responsible for MuAPI downtime, processing failures, incorrect results, hosted output availability, rate limits, account restrictions, billing, policy changes, or media-processing errors."
      ]
    },
    {
      "heading": "6. OpenAI and AI Processing",
      "paragraphs": [
        "The application may use OpenAI or other AI services for highlight ranking, text generation, title generation, hook generation, scoring, classification, or similar tasks. In the inspected implementation, local mode still may use OpenAI for the LLM highlight-ranking step.",
        "Information sent to AI providers may include transcript text, prompt text, source-related metadata, highlight-selection instructions, and other processing context.",
        "AI-generated outputs may be inaccurate, incomplete, biased, offensive, misleading, unexpected, duplicative, low quality, legally risky, or unsuitable for your intended use.",
        "You must verify all AI-generated outputs before relying on them or publishing them. We are not responsible for decisions, publications, losses, claims, takedowns, account penalties, monetization loss, or other consequences caused by AI-generated outputs."
      ]
    },
    {
      "heading": "7. Local AI Models and Local Processing",
      "paragraphs": [
        "The application may support local model processing, including faster-whisper or Whisper-style transcription models. Local processing performance depends on your hardware, operating system, model files, GPU drivers, CPU/GPU capability, memory, storage, installed dependencies, permissions, and configuration.",
        "We are not responsible for crashes, slow performance, failed processing, incorrect results, poor transcription quality, missing model files, corrupted model files, unsupported hardware, incompatible drivers, dependency conflicts, or unsupported local environments.",
        "[VERIFY: exact source, hosting provider, and license terms for each selectable local model]"
      ]
    },
    {
      "heading": "8. FFmpeg and Media Processing",
      "paragraphs": [
        "The application may use FFmpeg and similar media-processing tools for cutting, encoding, muxing, converting, reframing, and generating media files. FFmpeg is a third-party tool governed by its own license and behavior.",
        "[VERIFY: whether production releases bundle FFmpeg or require users to install it externally]",
        "[VERIFY: FFmpeg build configuration and license obligations for any distributed binaries]",
        "We are not responsible for:",
        "- FFmpeg errors or missing FFmpeg installations.",
        "- Missing codecs or unsupported input/output formats.",
        "- Failed downloads, cuts, conversions, encodes, muxes, or exports.",
        "- Corrupted outputs, partial outputs, quality loss, encoding artifacts, large file sizes, or audio/video desync.",
        "- Incorrect crops, poor face tracking, failed reframing, or unsuitable visual results.",
        "- Hardware acceleration problems, driver problems, or platform-specific media failures.",
        "You are responsible for checking all generated media files before publishing or distributing them."
      ]
    },
    {
      "heading": "9. User Content and Intellectual Property",
      "paragraphs": [
        "You retain whatever rights you already have in your own content. The application does not grant you rights to third-party videos, music, images, likenesses, voices, transcripts, datasets, models, fonts, APIs, software, or other protected material.",
        "You must only process content that you own or have permission to use. You are responsible for obtaining all required rights, permissions, licenses, consents, and clearances.",
        "You are responsible for copyright, licensing, privacy, publicity rights, creator rights, music rights, platform policies, and applicable law.",
        "We are not responsible for copyright claims, DMCA notices, takedowns, rejected uploads, demonetization, lost revenue, account strikes, account bans, platform enforcement, legal claims, or disputes caused by your source content, generated outputs, uploads, or publications."
      ]
    },
    {
      "heading": "10. Diagnostics, Paths, and Advanced Settings",
      "paragraphs": [
        "The application may provide diagnostics, dependency checks, API key profile settings, local model settings, output path controls, local file selection, crash draft handling, and advanced runtime information.",
        "Some settings and paths affect whether the application works correctly. If you manually change dependency paths, executable paths, FFmpeg paths, Python paths, model paths, output paths, working directories, environment variables, permissions, configuration files, diagnostics values, local files, or related settings, you are responsible for the consequences.",
        "We are not responsible if the application stops working, loses access to files, fails dependency checks, fails to process media, produces errors, or generates incorrect outputs because you changed advanced settings, paths, dependencies, permissions, environment variables, or local files.",
        "You should only change advanced settings if you understand the consequences. Reset, retry, recheck, or default options, if available, do not guarantee full recovery."
      ]
    },
    {
      "heading": "11. Dependencies and Local Environment",
      "paragraphs": [
        "The application may require external or bundled dependencies such as FFmpeg, Python, yt-dlp, OpenCV, faster-whisper, AI model files, GPU drivers, system libraries, runtime packages, platform-specific tools, network access, and provider accounts.",
        "Unless a dependency is explicitly bundled with your specific application build, you are responsible for installing, configuring, updating, and maintaining it.",
        "We are not responsible for unsupported operating systems, outdated drivers, antivirus or security software blocking files or processes, missing permissions, corrupted installs, conflicting software, missing runtime packages, dependency version conflicts, incompatible hardware, insufficient CPU/GPU/RAM/disk/network resources, or broken PATH/environment configuration."
      ]
    },
    {
      "heading": "12. Logs and Diagnostic Information",
      "paragraphs": [
        "Logs, crash drafts, diagnostics, output JSON, error messages, and support materials may include sensitive or identifying information, including file paths, dependency paths, local usernames embedded in paths, filenames, source URLs, prompts, transcripts, generated outputs, API errors, environment details, configuration values, stack traces, or processing status.",
        "You should review logs, diagnostics, screenshots, output JSON, and support bundles before sharing them publicly or with support.",
        "We are not responsible if you disclose sensitive information by sharing logs, diagnostics, screenshots, output files, generated JSON, crash drafts, or support materials."
      ]
    },
    {
      "heading": "13. Privacy and Data Handling",
      "paragraphs": [
        "Based on inspected repository behavior, the application supports both local processing and API-based processing. Local files and generated outputs may remain on your device unless you select a workflow that uses external APIs, license services, update services, or configured crash-report submission.",
        "External transmission may occur when:",
        "- You use MuAPI/API mode.",
        "- Local mode sends transcript or prompt context to OpenAI or another configured AI provider.",
        "- You activate, validate, or reset a license.",
        "- The licensing worker verifies a Gumroad purchase.",
        "- You check for or install updates.",
        "- You submit a crash report and a crash-report endpoint is configured.",
        "The application may store local project history, output metadata, settings, local model profile metadata, API key profile metadata, crash drafts, reset status, generated outputs, logs, configuration files, model caches, and license/session/device-related information.",
        "API key values are intended to be stored using operating-system credential storage when available, with a local fallback if credential storage fails. [VERIFY: production secret-storage behavior and fallback disclosure]",
        "No general telemetry or analytics SDK was identified during repository inspection. [VERIFY: telemetry, analytics, crash-report, and support-data behavior in production builds]",
        "You are responsible for deleting local outputs, downloaded media, generated clips, exported JSON, logs, caches, model files, and project history where the application or operating system allows deletion."
      ]
    },
    {
      "heading": "14. Licensing, Activation, and Device Binding",
      "paragraphs": [
        "The application may require license activation and validation. Licensing may include license keys, device binding, session validation, reset requests, Gumroad purchase verification, purchaser email records, server-side license records, and local license/session/device state.",
        "A license may not work if it is invalid, revoked, expired, already bound to another device, blocked, reset-pending, affected by payment issues, or rejected by the licensing backend.",
        "We are not responsible for access interruptions caused by invalid purchases, chargebacks, Gumroad issues, license server downtime, network failures, device reset delays, device changes, local storage corruption, or unsupported environments.",
        "[VERIFY: whether refund terms, including any 7-day manual refund policy, should be included in this document]"
      ]
    },
    {
      "heading": "15. Third-Party Licenses",
      "paragraphs": [
        "Third-party libraries, APIs, services, tools, AI models, datasets, and other components remain governed by their own licenses and terms.",
        "These may include FFmpeg, Python packages, Rust crates, Node/pnpm packages, Tauri components, Svelte/Vite tooling, OpenCV, yt-dlp, faster-whisper, Whisper-style model files, OpenAI, MuAPI, Gumroad, Cloudflare, and other dependencies or services.",
        "You are responsible for complying with applicable third-party terms where relevant, especially if you redistribute the application, bundle dependencies, distribute model files, use outputs commercially, or process third-party content.",
        "[VERIFY: complete third-party notices, open-source notices, FFmpeg notices, and model license notices before release]"
      ]
    },
    {
      "heading": "16. Prohibited Uses",
      "paragraphs": [
        "You must not use the application to:",
        "- Violate any law or regulation.",
        "- Infringe copyright, trademark, privacy, publicity, or other rights.",
        "- Process private, confidential, or sensitive content without permission.",
        "- Harass, abuse, threaten, defame, exploit, or harm others.",
        "- Create or distribute malware or harmful automation.",
        "- Generate spam, deceptive content, or fraudulent content.",
        "- Manipulate platforms, rankings, engagement, recommendations, or monetization systems.",
        "- Scrape, download, or process content without authorization.",
        "- Violate API provider terms or platform terms.",
        "- Misuse API keys, credentials, license keys, or accounts.",
        "- Bypass, tamper with, disable, or interfere with licensing, activation, device binding, security controls, or access controls except where applicable law expressly permits."
      ]
    },
    {
      "heading": "17. No Warranty",
      "paragraphs": [
        "The application is provided \"as is\" and \"as available.\" To the maximum extent permitted by law, we disclaim all warranties, whether express, implied, statutory, or otherwise, including warranties of merchantability, fitness for a particular purpose, title, non-infringement, accuracy, compatibility, availability, and uninterrupted operation.",
        "We do not guarantee that the application will be error-free, uninterrupted, secure, compatible with your environment, accepted by any platform, monetized, ranked, visible, profitable, or suitable for your intended purpose.",
        "We do not guarantee output correctness, transcript accuracy, highlight quality, title quality, hook quality, media quality, API availability, dependency compatibility, local model performance, FFmpeg success, diagnostics accuracy, or update availability."
      ]
    },
    {
      "heading": "18. Limitation of Liability",
      "paragraphs": [
        "To the maximum extent permitted by law, we are not liable for indirect, incidental, special, consequential, exemplary, punitive, or similar damages.",
        "We are not liable for losses or claims involving data loss, lost revenue, lost profits, account bans, account strikes, API costs, unexpected charges, publishing mistakes, copyright claims, takedowns, rejected uploads, failed uploads, demonetization, corrupted outputs, damaged media files, failed conversions, AI mistakes, local processing errors, dependency failures, third-party service failures, license activation issues, device reset delays, unsupported environments, user-modified settings, user-modified paths, or disclosed diagnostic information.",
        "[VERIFY: whether to include a monetary liability cap, such as fees paid in the previous 12 months]"
      ]
    },
    {
      "heading": "19. Updates and Changes",
      "paragraphs": [
        "The application may be updated over time. Updates may add, remove, or change features, APIs, supported providers, dependencies, model options, output formats, diagnostics behavior, settings, licensing behavior, supported operating systems, security behavior, or system requirements.",
        "We do not guarantee that any specific feature, provider, dependency, model, workflow, or output format will remain available.",
        "These Terms may also be updated over time. Continued use of the application after updated Terms become effective means you accept the updated Terms."
      ]
    },
    {
      "heading": "20. Termination",
      "paragraphs": [
        "We may suspend or terminate access to the application or licensing services if we reasonably believe that you violated these Terms, misused a license key, bypassed access controls, reversed payment, used the application unlawfully, harmed third parties, or created legal, security, or operational risk.",
        "You may stop using the application at any time. Termination does not remove your responsibility for content you created, processed, uploaded, published, or distributed while using the application."
      ]
    },
    {
      "heading": "21. Contact Information",
      "paragraphs": [
        "[DEVELOPER NAME]",
        "[CONTACT EMAIL]",
        "[WEBSITE OR SUPPORT URL]"
      ]
    },
    {
      "heading": "22. Governing Law",
      "paragraphs": [
        "These Terms are governed by the laws of [JURISDICTION], without regard to conflict-of-law rules.",
        "[VERIFY: governing law, venue, arbitration, class-action waiver, consumer-law requirements, and regional compliance obligations with qualified legal counsel]"
      ]
    }
  ],
  "privacy": [
    {
      "heading": "Privacy Policy",
      "paragraphs": [
        "Last updated: May 23, 2026",
        "This Privacy Policy explains how [APP NAME] handles data based on the currently inspected application behavior. Repository evidence refers to the project as \"AI YouTube Shorts Generator\" and some application UI references \"Signal Forge.\" [VERIFY: production app name and legal product name]",
        "[APP NAME] is provided by [DEVELOPER NAME]. Contact: [CONTACT EMAIL].",
        "This document should be reviewed by a qualified lawyer before public release."
      ]
    },
    {
      "heading": "1. Processing Overview",
      "paragraphs": [
        "The application supports both local processing and API-based processing.",
        "Local processing may occur on your device when the application processes local files, stores project history, stores settings, checks dependencies, runs local media tools, uses local model files, creates output files, or stores crash drafts and diagnostics.",
        "External processing may occur when you use API mode, configure third-party API keys, activate or validate a license, request a device reset, check for updates, install updates, or submit crash information where a crash-report endpoint is configured."
      ]
    },
    {
      "heading": "2. Data You Provide or Generate",
      "paragraphs": [
        "Depending on your use, the application may handle:",
        "- YouTube URLs or other source references.",
        "- Local video files and downloaded source media.",
        "- Audio, video, transcript text, and generated clips.",
        "- AI prompts or prompt-like processing context.",
        "- Generated titles, hooks, scores, reasons, metadata, and JSON output.",
        "- Output file paths and local project history.",
        "- API keys, API key profile labels, and masked key metadata.",
        "- License keys, masked license information, activation status, reset request state, and device-related license information.",
        "- Logs, crash drafts, diagnostics, error messages, dependency paths, environment details, and configuration values."
      ]
    },
    {
      "heading": "3. Local Storage",
      "paragraphs": [
        "The application may store data locally on your device, including project history, generated output metadata, settings, API key profile metadata, local model profile metadata, model caches, logs, crash drafts, reset status, configuration files, generated media files, exported JSON, and license/session/device-related information.",
        "API key values are intended to be stored using operating-system credential storage when available, with a local fallback if credential storage fails. [VERIFY: production secret-storage behavior and fallback disclosure]",
        "You are responsible for managing and deleting local files, generated outputs, downloaded media, exported JSON, logs, caches, model files, and project history where the application or operating system allows deletion."
      ]
    },
    {
      "heading": "4. Data Sent to Third Parties",
      "paragraphs": [
        "The application may send data to third parties when you choose or configure features that require them.",
        "MuAPI/API mode may receive source URLs, media references, transcript-related data, prompt-like processing data, highlight data, timing data, aspect-ratio settings, and other processing information needed to generate clips.",
        "OpenAI or another configured AI provider may receive transcript text, prompt text, highlight-selection instructions, generated context, and related metadata for AI processing.",
        "License services may receive license activation, validation, reset, device binding, and session information. The licensing backend may store hashed license keys, purchaser email, provider sale identifiers, device binding information, reset request information, audit events, and related metadata.",
        "Gumroad-related verification may be used by the licensing backend to confirm purchase records.",
        "Updater services may receive information needed to check for and install application updates. [VERIFY: production update endpoint and signing configuration]",
        "Crash-report services may receive crash drafts or diagnostic information only if a crash-report endpoint is configured and the user submits a report. [VERIFY: production crash-report endpoint, submission behavior, and retention policy]"
      ]
    },
    {
      "heading": "5. Logs and Diagnostics",
      "paragraphs": [
        "Logs, diagnostics, crash drafts, output JSON, and support materials may include sensitive information such as local file paths, dependency paths, usernames embedded in paths, filenames, source URLs, transcript text, prompt text, generated outputs, API errors, environment details, configuration values, processing status, and stack traces.",
        "Review logs and diagnostics before sharing them publicly or with support. We are not responsible if you disclose sensitive information by sharing logs, screenshots, crash drafts, output JSON, or diagnostic details."
      ]
    },
    {
      "heading": "6. Telemetry, Analytics, and Crash Reports",
      "paragraphs": [
        "No general telemetry or analytics SDK was identified during repository inspection.",
        "The application includes crash draft behavior and may support user-submitted crash reports if an endpoint is configured.",
        "[VERIFY: whether production builds collect telemetry, analytics, automatic crash reports, or support diagnostics beyond the inspected behavior]"
      ]
    },
    {
      "heading": "7. API Keys and Credentials",
      "paragraphs": [
        "You are responsible for protecting your API keys, credentials, license keys, and accounts.",
        "Do not share API keys, license keys, logs, screenshots, support bundles, or configuration files that expose secrets or sensitive account information.",
        "We are not responsible for unauthorized charges, revoked access, leaked credentials, misconfigured API keys, provider account restrictions, or losses caused by compromised credentials."
      ]
    },
    {
      "heading": "8. User Content and Rights",
      "paragraphs": [
        "You are responsible for ensuring you have permission to process any video, audio, transcript, image, text, likeness, voice, or other content you provide to the application.",
        "The application does not grant you rights to third-party content, music, videos, images, datasets, models, APIs, or platform content."
      ]
    },
    {
      "heading": "9. Third-Party Services",
      "paragraphs": [
        "Third-party services have their own privacy policies, terms, data practices, retention rules, and security practices. Review the policies for MuAPI, OpenAI, Gumroad, Cloudflare, and any other provider you use with the application.",
        "We do not control third-party providers and are not responsible for their data practices."
      ]
    },
    {
      "heading": "10. Data Retention and Deletion",
      "paragraphs": [
        "Local data may remain on your device until you delete it, the application deletes it, or your operating system removes it.",
        "User data deletion removes or anonymizes application-controlled licensing records. Historical idempotency records may not always be linkable to a deletion subject and are handled on a best-effort basis.",
        "Cloudflare platform logs and provider-level infrastructure logs are governed by the relevant provider or account retention policy and are not directly deleted by the application.",
        "Server-side license, purchase, device binding, reset, audit, support, update, or crash-report data may be retained according to the relevant backend or provider configuration.",
        "[VERIFY: production retention and deletion policy for license records, purchaser email, device binding information, support requests, crash reports, and logs]"
      ]
    },
    {
      "heading": "11. Security",
      "paragraphs": [
        "The application uses local and provider-specific mechanisms for storage and processing, including operating-system credential storage where available for secrets. However, no system can be guaranteed secure.",
        "You are responsible for securing your device, operating system account, API keys, license keys, local files, generated outputs, logs, and configuration.",
        "Do not use the application on a device or account you do not trust."
      ]
    },
    {
      "heading": "12. Children and Sensitive Content",
      "paragraphs": [
        "The application is not intended for use by children. [VERIFY: minimum user age and child-directed service policy]",
        "Do not process private, confidential, sensitive, biometric, health, financial, child-related, or legally protected content unless you have all required rights, permissions, consents, and legal basis."
      ]
    },
    {
      "heading": "13. Changes to This Privacy Policy",
      "paragraphs": [
        "This Privacy Policy may be updated over time to reflect changes in the application, services, dependencies, data handling, or legal requirements.",
        "Continued use of the application after changes become effective means you accept the updated policy."
      ]
    },
    {
      "heading": "14. Contact",
      "paragraphs": [
        "[DEVELOPER NAME]",
        "[CONTACT EMAIL]",
        "[WEBSITE OR SUPPORT URL]"
      ]
    }
  ],
  "compliance": [
    {
      "heading": "Data Compliance",
      "paragraphs": [
        "Last updated: May 25, 2026",
        "Effective date: [EFFECTIVE DATE]",
        "Product: [APP NAME] / AI YouTube Shorts Generator / Signal Forge [VERIFY FINAL PRODUCT NAME]",
        "Operator: [LEGAL COMPANY / DEVELOPER NAME]",
        "Address: [COMPANY ADDRESS]",
        "Privacy contact: [PRIVACY EMAIL]",
        "Support contact: [SUPPORT EMAIL]",
        "This document is a practical compliance guide based on the inspected application codebase and is intended to support privacy, security, and legal review before release. It is not legal advice. Exact obligations depend on the operator's location, user locations, business size, revenue, data volume, payment model, processor contracts, and final production configuration. A qualified lawyer should review this document before commercial release."
      ]
    },
    {
      "heading": "1. Purpose and Scope",
      "paragraphs": [
        "This document describes data categories, data flows, privacy expectations, security controls, and release-readiness items for the desktop application, local runtime, API processing, licensing backend, payment-provider verification, admin console, diagnostics, and optional crash-report workflow.",
        "It focuses on United States and European Union privacy/data-protection expectations, with notes for UK GDPR where appropriate."
      ]
    },
    {
      "heading": "2. System Overview",
      "paragraphs": [
        "The inspected application includes:",
        "- A Tauri/Rust desktop app with Svelte UI.",
        "- A Python bridge and Python media-processing pipeline.",
        "- API mode using MuAPI for hosted video processing.",
        "- Local mode using yt-dlp, faster-whisper, OpenAI, FFmpeg, and OpenCV.",
        "- Local runtime-pack and local model download flows.",
        "- License activation, validation, session state, device binding, and reset requests.",
        "- A Cloudflare Worker/D1 licensing backend.",
        "- Gumroad purchase verification.",
        "- A separate admin desktop console for reset review and license/device/audit visibility.",
        "- Local project history, diagnostics, logs, crash-report drafts, and optional output JSON."
      ]
    },
    {
      "heading": "3. Data Inventory",
      "paragraphs": [
        "Source references: YouTube URLs and local video paths may appear in local UI/state, MuAPI in API mode, or yt-dlp/platforms in local URL mode. URLs and paths may identify users, creators, projects, or private files.",
        "Source media: downloaded videos, local videos, and audio/video frames may be stored locally in local mode or handled by MuAPI in API mode. Treat this as user content that may contain personal or sensitive data.",
        "Transcripts: text segments, timestamps, and duration may be stored in local memory/output JSON or sent to MuAPI/OpenAI depending on mode. Transcript text can contain personal data or sensitive data.",
        "Prompts: content classification and highlight prompts may be sent to MuAPI in API mode or OpenAI in local mode. Prompt text includes transcript excerpts and generation instructions.",
        "Generated outputs: clips, titles, hooks, scores, virality reasons, and clip URLs/paths may exist in the local project library, local output files, output JSON, or MuAPI hosted URLs. Users must review outputs before publication.",
        "Project history: project names, sourceUrl, short metadata, and updatedAt values may be stored in localStorage and can include URLs or local paths.",
        "API credentials: MuAPI and OpenAI keys may be stored in OS credential storage and/or local fallback storage. Fallback storage is sensitive.",
        "Admin credentials: Worker base URL and admin API token may be stored in OS credential storage and/or local fallback storage. Admin tokens grant backend visibility/actions.",
        "License credentials: license keys and access tokens are sensitive and are not intended for logs or display.",
        "License records: hashed license key, entitlement, provider, and sale ID may be stored in Cloudflare D1. Server records store hashes rather than raw license keys.",
        "Purchaser data: purchaser email and Gumroad sale/product IDs may exist in Gumroad and Worker/D1. Email is personal data. Admin API masks emails in several responses.",
        "Device data: device public key, local private key material, device ID, and fingerprint JSON may exist in local app data and Worker/D1. Device fingerprint may include OS/platform/arch and hashed hostname where collected.",
        "Reset data: reset request ID, status, masked license, purchaser email, and timestamps may exist in local reset cache, Worker/D1, and the admin console.",
        "Audit/idempotency: event type, actor, metadata JSON, payload hash, and response body may exist in Worker/D1. Response bodies may include sensitive activation/reset response data, and retention policy is not fully implemented in code.",
        "Logs/diagnostics: runtime paths, dependency status, model errors, debug refs, and stack traces may exist in local logs/localStorage and optional crash endpoints. They may include local usernames in paths and source details.",
        "Runtime/model data: runtime-pack status, model profile, cache paths, and model files may exist in local app data and runtime/model hosts. Downloaded files may have separate licenses."
      ]
    },
    {
      "heading": "4. Personal Data Categories",
      "paragraphs": [
        "The application may process personal data, including:",
        "- Names or identifiers embedded in local file paths, project names, media filenames, URLs, transcripts, or generated content.",
        "- Purchaser email addresses received from Gumroad or reset workflows.",
        "- Provider sale IDs and product IDs tied to purchases.",
        "- License status and entitlement records associated with a purchaser.",
        "- Device identifiers, public keys, device binding records, app version, OS/platform/architecture, and hashed hostname values.",
        "- Local IP/network metadata visible to third-party APIs, update hosts, runtime-pack hosts, model hosts, Gumroad, Cloudflare, or source platforms through normal network requests.",
        "- Admin/support actions tied to reset decisions and audit events.",
        "- Crash-report draft contents if submitted."
      ]
    },
    {
      "heading": "5. Sensitive or Security-Relevant Data",
      "paragraphs": [
        "The application may handle sensitive or security-relevant data even when it is not legally classified as sensitive personal information:",
        "- License keys.",
        "- Access tokens.",
        "- MuAPI API keys.",
        "- OpenAI API keys.",
        "- Admin API tokens.",
        "- Worker secrets configured by the operator.",
        "- Device private key material stored locally.",
        "- Local fallback secret files.",
        "- User media that may contain faces, voices, children, biometrics, health information, financial information, or other sensitive content.",
        "- Transcripts and prompts that may reveal private or confidential information.",
        "- Logs and diagnostics containing paths, runtime details, source URLs, and stack traces.",
        "Policy requirement: do not request or encourage users to process sensitive or special-category content unless the operator has a documented legal basis, consent process, security model, and support workflow."
      ]
    },
    {
      "heading": "6. Data Flow Overview",
      "paragraphs": [
        "Activation and license validation: the user enters a license key after accepting Terms. The desktop app sends the license key, device public key, device fingerprint, app version, and timestamp to the license Worker. The Worker normalizes and hashes the license key with an operator-configured hash pepper, checks D1 license records, writes or updates device binding records, and returns a signed access token, masked license key, entitlement, bound device details, and token expiration. The desktop stores local license/session/device state using OS credential storage and local fallback mechanisms. Later session validation sends the access token to the Worker.",
        "Gumroad purchase verification: Gumroad sends a webhook form payload containing sale ID, product ID, and purchaser email. The Worker calls Gumroad sales API with an operator-configured access token, verifies sale ID, product ID, email, refund status, dispute status, and license key availability, then hashes the verified license key and stores license entitlement, purchaser email, provider, provider sale ID, and audit metadata in D1.",
        "Device reset workflow: the user requests a device reset with a license key from the activation screen or settings. The desktop sends license context, device public key, fingerprint, app version, and timestamp to the Worker. The Worker stores a reset request with hashed license key where available, masked license key, optional purchaser email, status, and timestamps. User or app checks reset status by request ID. Admin console lists reset requests and can approve or reject pending requests with an admin token. On approval, the Worker deactivates active device bindings for the license hash and records audit/idempotency data.",
        "API generation mode: the user provides a URL and generation settings. The desktop sends source URL and settings to MuAPI. MuAPI downloads or hosts media and returns hosted URLs. The desktop sends hosted media URL to MuAPI transcription, sends transcript-derived prompts to a MuAPI LLM endpoint, sends crop timestamps and aspect ratio to MuAPI autocrop, and receives transcripts, highlights, clip URLs, and provider payloads. The user may store project history locally and optionally export full output JSON.",
        "Local generation mode: the user provides a local file path or URL. Local files are processed locally. URLs may be downloaded through yt-dlp. faster-whisper transcribes audio locally using local model files. OpenAI receives transcript-derived prompts for highlight selection. FFmpeg and OpenCV create local clips and reframe outputs. Generated media and metadata are stored locally and optionally exported to JSON.",
        "Diagnostics and crash drafts: the app checks runtime/dependency status and displays local paths and status. Local model download failures may write diagnostic logs. Frontend errors create local crash drafts with selected redaction. Crash drafts are submitted only if a crash endpoint is configured and the user chooses to submit."
      ]
    },
    {
      "heading": "7. Local Processing vs Cloud/API Processing",
      "paragraphs": [
        "Local processing claims must be precise. The inspected local mode performs transcription and media rendering locally, but local mode may still use network services for:",
        "- OpenAI prompt processing.",
        "- yt-dlp downloads from YouTube or other platforms.",
        "- Runtime-pack downloads.",
        "- faster-whisper/model downloads.",
        "- License activation, validation, and reset.",
        "- Update checks and installs.",
        "- Optional crash-report submission.",
        "Do not market the application as fully offline unless a separate build disables or replaces all network-dependent workflows and this is verified."
      ]
    },
    {
      "heading": "8. Licensing Backend Data Controls",
      "paragraphs": [
        "Designed safeguards identified in code:",
        "- Server-side license records use hashed license keys rather than raw license keys.",
        "- License hashes use an operator-configured hash pepper.",
        "- Access tokens are signed with an operator-configured token-signing secret.",
        "- Admin API requires a bearer admin token.",
        "- Admin endpoints return masked purchaser emails, hash prefixes, public-key prefixes, and summaries for several views.",
        "- Audit events and idempotency records support review and replay handling.",
        "Open items:",
        "- Define backend retention periods.",
        "- Define data deletion/anonymization process.",
        "- Verify admin token rotation, storage, logging, and least-privilege controls.",
        "- Verify rate limits and abuse protection for public Worker routes.",
        "- Verify D1 backup/export/deletion procedures.",
        "- Verify production HTTPS-only configuration."
      ]
    },
    {
      "heading": "9. Admin App Data Access",
      "paragraphs": [
        "The admin desktop console can access licensing operational data through authenticated Worker APIs. Admin access includes operational counts, reset requests, masked license/purchaser data, device binding summaries, audit summaries, and idempotency summaries.",
        "Compliance requirements:",
        "- Restrict admin tokens to authorized personnel only.",
        "- Use role-based access or separate tokens if multiple admin roles are introduced.",
        "- Log admin decisions without storing unnecessary free-form personal data.",
        "- Train admins not to copy purchaser emails, license hashes, device IDs, or reset data into unapproved systems.",
        "- Provide a documented reset approval/rejection policy.",
        "- Rotate admin tokens after personnel changes or suspected exposure."
      ]
    },
    {
      "heading": "10. API Keys and User-Provided Credentials",
      "paragraphs": [
        "Users may provide MuAPI and OpenAI API keys. Admins may provide Worker admin tokens.",
        "Expected handling:",
        "- Use password fields in UI.",
        "- Store profile metadata separately from secret values.",
        "- Display only last-four or redacted tokens where possible.",
        "- Avoid putting full keys in localStorage, logs, errors, screenshots, tests, or support bundles.",
        "- Allow users to delete API-key profiles.",
        "Risk note: inspected local fallback secret mechanisms can store secrets in local files. Release documentation should clearly tell users to secure their device account and avoid sharing app-data folders."
      ]
    },
    {
      "heading": "11. Logs and Diagnostics",
      "paragraphs": [
        "Logs and diagnostics should be treated as potentially sensitive because they may include full local paths and usernames, source URLs, model/cache paths, Python and bridge paths, runtime status and platform details, error messages and stack traces, and transcript/prompt/provider fragments in some failures.",
        "Compliance expectations:",
        "- Do not upload logs automatically without opt-in.",
        "- Redact license keys, API keys, admin tokens, access tokens, raw emails, and machine identifiers where practical.",
        "- Provide user-visible review before support submission where possible.",
        "- Document retention and deletion for submitted crash/support records."
      ]
    },
    {
      "heading": "12. GDPR Considerations",
      "paragraphs": [
        "The operator may be a controller for licensing records, payment-provider metadata, support/admin decisions, crash reports, and user relationship data. Depending on configuration, the operator may also be a controller or processor for user media and generated content processed through hosted services.",
        "The operator should complete Records of Processing Activities where required, Data Processing Agreements with processors/subprocessors, Transfer Impact Assessments for international transfers where required, Legitimate Interests Assessments for fraud prevention, license enforcement, security logging, and diagnostics, and a Data Protection Impact Assessment if processing is likely high risk, such as large-scale sensitive media, biometric/face/voice data, children's data, or systematic monitoring."
      ]
    },
    {
      "heading": "13. GDPR Legal Bases",
      "paragraphs": [
        "Potential lawful bases to document include contract performance for license activation and validation; contract performance and legitimate interests for device binding and reset workflow; contract performance, legal obligation, and legitimate interests for Gumroad sale verification; contract performance or user instruction for generation requested by the user; contract performance or user instruction for API-key storage; consent or legitimate interests for crash-report submission; legitimate interests for logs and security audit events; legal obligation and legitimate interests for legal/tax/chargeback records; and consent or legitimate interests depending on region for marketing if added later.",
        "Do not claim GDPR compliance as guaranteed. State that the application is designed to support compliance when operated with appropriate contracts, settings, retention schedules, and request-handling procedures."
      ]
    },
    {
      "heading": "14. GDPR Data Subject Rights",
      "paragraphs": [
        "Where GDPR applies, users may have rights to access personal data, receive information about processing, correct inaccurate data, delete data subject to exceptions, restrict processing, object to processing based on legitimate interests, receive portable data in certain cases, withdraw consent where processing is based on consent, and lodge a complaint with a supervisory authority.",
        "Operational requirements include providing a privacy request channel at [PRIVACY EMAIL], verifying identity before acting on license/payment/device records, defining response timelines and escalation owners, identifying records that can be deleted, anonymized, corrected, or must be retained, and documenting exceptions for fraud prevention, payment disputes, tax/accounting, security, legal claims, and licensing enforcement."
      ]
    },
    {
      "heading": "15. ePrivacy and Terminal-Equipment Considerations",
      "paragraphs": [
        "The desktop app stores data on the user's device using local files, OS credential stores, and desktop web storage. In the EU/EEA, ePrivacy-style rules may apply to storing or accessing information on a user's terminal equipment.",
        "Storage necessary for requested app functionality, licensing, security, settings, and local project history may be treated differently from nonessential analytics or advertising storage. No advertising or analytics SDK was identified. If analytics, telemetry, marketing pixels, or nonessential tracking are added later, obtain appropriate consent and update policies."
      ]
    },
    {
      "heading": "16. UK GDPR Note",
      "paragraphs": [
        "For UK users, UK GDPR and the Data Protection Act 2018 may apply. The operator should assess UK representative requirements if the operator is outside the UK, UK International Data Transfer Agreement or UK Addendum for transfers, ICO registration and fee requirements, and UK-specific privacy notice language and consumer-law requirements."
      ]
    },
    {
      "heading": "17. CCPA/CPRA-Style US Privacy Considerations",
      "paragraphs": [
        "If the operator is subject to the CCPA/CPRA or similar US state privacy laws, the operator should provide rights to know/access categories and specific pieces of personal information, delete personal information subject to exceptions, correct inaccurate personal information, opt out of sale/share or targeted advertising where applicable, limit use/disclosure of sensitive personal information where applicable, appeal denied requests where required by state law, and avoid discriminatory treatment for rights exercise.",
        "No sale, sharing for cross-context behavioral advertising, or targeted advertising workflow was identified in the inspected code. The operator must verify this for final production, especially if analytics, ads, affiliate tracking, referral tracking, or marketing integrations are added."
      ]
    },
    {
      "heading": "18. General US State Privacy Considerations",
      "paragraphs": [
        "Other US state privacy laws may require clear notices at or before collection, data minimization and purpose limitation, consumer rights request workflows, processor contracts, reasonable security safeguards, sensitive data consent or opt-out controls, and data protection assessments for high-risk processing.",
        "Applicability depends on thresholds such as revenue, number of consumers, percentage of revenue from data sale, and data categories processed."
      ]
    },
    {
      "heading": "19. Data Minimization",
      "paragraphs": [
        "Recommended minimization controls:",
        "- Store only hashed license keys on the backend.",
        "- Avoid storing raw purchaser emails in audit metadata unless necessary.",
        "- Avoid storing full fingerprints where summaries or hashes are enough.",
        "- Avoid including transcripts, prompts, or source URLs in logs unless necessary for support.",
        "- Keep local project history limited to what the library UI requires.",
        "- Make output JSON optional and user-selected.",
        "- Do not upload crash drafts automatically.",
        "- Redact local paths and usernames in support exports where possible.",
        "- Avoid retaining idempotency response bodies longer than necessary because activation replay records may contain signed access tokens or device-binding response data."
      ]
    },
    {
      "heading": "20. Purpose Limitation",
      "paragraphs": [
        "Data collected for one purpose should not be reused for unrelated purposes without a valid legal basis and updated notice.",
        "Examples: purchaser email should be used for purchase, support, license, reset, refund, fraud-prevention, and legal/accounting purposes, not unrelated marketing unless separately permitted. Device binding data should be used for license enforcement, security, reset workflow, and support, not unrelated tracking. User media/transcripts/prompts should be used only to provide the requested generation workflow and support chosen by the user. Crash reports should be used for debugging and support, not profiling or marketing."
      ]
    },
    {
      "heading": "21. Retention Principles",
      "paragraphs": [
        "The inspected code does not implement a complete backend retention schedule. Before release, define and document retention periods.",
        "Recommended retention approach: local files are user-controlled and should have clear deletion instructions. Crash drafts are local until dismissed/submitted, and submitted reports should be retained for [RETENTION PERIOD]. License records should be retained while the license is active and for [RETENTION PERIOD] after expiration/refund/termination as needed for legal, fraud, and accounting purposes. Device bindings should be retained while needed for license enforcement and reset history, then deleted or anonymized after [RETENTION PERIOD]. Reset requests should be retained while pending and for [RETENTION PERIOD] after decision for audit/fraud purposes. Audit events should be retained for [RETENTION PERIOD] consistent with security and legal requirements. Idempotency records should be retained short term unless needed for disputes. Gumroad/payment metadata should be retained according to tax/accounting and payment-dispute requirements."
      ]
    },
    {
      "heading": "22. User Access, Deletion, and Correction Requests",
      "paragraphs": [
        "The operator should implement a documented workflow for receiving requests at [PRIVACY EMAIL], verifying identity using license key, purchase email, sale ID, or other safe evidence without requesting excessive data, exporting relevant backend records in a human-readable form, correcting purchaser email or entitlement metadata where appropriate, deleting or anonymizing eligible records, preserving records required for chargebacks, fraud prevention, tax, legal claims, or security, and recording request outcome and date.",
        "The inspected desktop app provides local controls for some local data, but backend privacy rights require operator-side processes."
      ]
    },
    {
      "heading": "23. Security Measures",
      "paragraphs": [
        "Security controls identified or expected include license-gated UI for generation features, device binding and signed access tokens, server-side hashed license keys with hash pepper, admin bearer-token authentication, masked emails/license keys in several UI/API responses, local secure-store use where available, crash-draft redaction for selected secret/license patterns, runtime-pack checksum validation, structured error mapping to avoid exposing raw auth failures, and avoidance of automatic telemetry/analytics in inspected code.",
        "Additional release controls recommended: enforce HTTPS for hosted Worker/update/runtime endpoints, set and rotate strong Worker secrets, add rate limiting and abuse controls to public Worker routes, implement admin token rotation and least privilege, review Tauri Content Security Policy and updater signing before production, encrypt or eliminate plaintext local fallback secrets where feasible, add server-side retention/deletion tooling, and run dependency/license/security audits before each release."
      ]
    },
    {
      "heading": "24. Breach Response Considerations",
      "paragraphs": [
        "Before release, the operator should create an incident response plan covering suspected exposure of Worker secrets, Gumroad tokens, admin tokens, API keys, license keys, access tokens, or D1 data; compromised runtime-pack/update hosting; malicious or corrupted runtime/model downloads; unauthorized admin access; accidental support log or crash-report disclosure; user notification and regulatory notification timelines under applicable law; and token revocation, secret rotation, forensic preservation, and post-incident remediation."
      ]
    },
    {
      "heading": "25. International Transfer Considerations",
      "paragraphs": [
        "Data may move across borders through Cloudflare, Gumroad, MuAPI, OpenAI, update hosts, runtime-pack hosts, model hosts, crash-report endpoints, and support operations.",
        "For EU/EEA or UK data, assess processor locations, subprocessor lists, Standard Contractual Clauses or UK transfer mechanisms, Transfer Impact Assessments, data localization or regional processing options, and whether user media/transcripts are sent to third countries."
      ]
    },
    {
      "heading": "26. Processor and Subprocessor Table",
      "paragraphs": [
        "Cloudflare Workers/D1: processor or infrastructure provider for license hashes, emails, device bindings, reset data, audit/idempotency records, and licensing backend behavior. Triggered by activation, validation, reset, and admin actions. DPA and retention review required.",
        "Gumroad: independent controller or processor depending terms for sale ID, product ID, purchaser email, license key, refund/dispute signals, and purchase/license verification. Terms/DPA/refund review required.",
        "MuAPI: processor or independent provider depending terms for URLs, hosted media references, transcripts, prompts, crop settings, generated clip URLs, and API-mode generation. Terms/DPA/content retention review required.",
        "OpenAI: processor or provider under API terms for transcript-derived prompts and processing context used for local-mode highlight ranking. API terms/data-use review required.",
        "YouTube/Google and other platforms: independent controllers/platforms for URL requests, media access, network metadata, and source media access/download. Platform terms compliance required.",
        "Model hosts such as Hugging Face: processor or independent provider depending use for model download request metadata and model files. Model license and host terms review required.",
        "Runtime-pack host: processor/infrastructure provider for platform/arch request metadata and runtime archive downloads. Host, signing, and retention review required.",
        "Update host: processor/infrastructure provider for app version, platform, arch, and update request metadata. Endpoint/signing review required.",
        "Crash-report endpoint: processor/support provider for crash draft, app version, platform, stack/error, and user-submitted diagnostics. Provider/retention review required.",
        "Admin operators/support staff: internal recipients for reset, license, device, audit summaries, and backend records if directly accessed. Access policy and training required."
      ]
    },
    {
      "heading": "27. Sensitive-Data Handling",
      "paragraphs": [
        "The app should not be positioned as designed for regulated sensitive data. Users should be instructed not to process private, confidential, health, biometric, financial, child-related, or special-category content unless they have all required rights and legal bases.",
        "If the operator decides to support sensitive-data use cases, complete a separate DPIA/security review, strengthen encryption and retention controls, define support restrictions, add processor terms, and update all policies."
      ]
    },
    {
      "heading": "28. Children's Data",
      "paragraphs": [
        "The app is not intended for children. Before release, define the minimum age and child-directed-service position.",
        "If users may process videos containing children, the operator should disclose that user media may contain children's personal data and require users to obtain all required rights and consents. Do not knowingly collect children's data for licensing/support accounts without a compliant parental-consent process if required."
      ]
    },
    {
      "heading": "29. AI-Generated Content and User Responsibility",
      "paragraphs": [
        "AI outputs can be incorrect, biased, offensive, misleading, infringing, or unsuitable for publication. The user remains responsible for reviewing outputs and ensuring they have rights to source media and generated content.",
        "Compliance implications: do not claim AI outputs are legally cleared, do not claim platform monetization or acceptance is guaranteed, disclose cloud AI processing where applicable, and provide user responsibility language for copyright, publicity rights, privacy rights, and platform rules."
      ]
    },
    {
      "heading": "30. Compliance Checklist Before Release",
      "paragraphs": [
        "Complete these items before commercial release:",
        "- Fill in company name, address, privacy email, support email, website, governing law, and effective dates.",
        "- Have qualified counsel review Terms, Privacy Policy, Third-Party Notices, Data Compliance, refund policy, consumer terms, and regional notices.",
        "- Sync in-app policy content with the final markdown policy documents if the app displays policy text internally.",
        "- Verify final production product name across UI, bundle metadata, policies, purchase pages, and support materials.",
        "- Generate npm, Cargo, and Python dependency license reports.",
        "- Verify FFmpeg, yt-dlp, Python runtime, runtime-pack, and model redistribution obligations.",
        "- Verify model licenses for all selectable faster-whisper/Whisper models.",
        "- Confirm MuAPI, OpenAI, Gumroad, Cloudflare, update host, runtime-pack host, model host, and crash-report provider terms and DPAs.",
        "- Define backend retention periods and implement deletion/anonymization tooling.",
        "- Document privacy request intake, identity verification, fulfillment, denial, and appeal process.",
        "- Confirm no analytics/telemetry/remote logging has been added without policy updates and consent where required.",
        "- Review Worker authentication, secrets, admin token rotation, rate limits, abuse controls, and audit logging.",
        "- Review Tauri CSP, permissions/capabilities, updater signing/public key, and runtime-pack verification.",
        "- Verify crash-report redaction and user-submitted-only behavior.",
        "- Document support access rules and admin reset decision policy.",
        "- Create incident response and breach-notification playbooks.",
        "- Confirm payment/refund/chargeback policy with Gumroad and consumer-law counsel."
      ]
    },
    {
      "heading": "31. Open Legal and Compliance Items",
      "paragraphs": [
        "The following items require legal or operator decisions:",
        "- Final legal entity and contact information.",
        "- Final governing law, venue, arbitration, class-action waiver, and consumer-law carveouts.",
        "- Final refund policy and regional consumer cancellation rights.",
        "- Data Processing Agreements and subprocessor disclosures.",
        "- Retention periods for all backend and support records.",
        "- Whether the operator is subject to GDPR, UK GDPR, CCPA/CPRA, other US state privacy laws, or other regional laws.",
        "- Whether a DPO, EU representative, UK representative, or privacy registration is required.",
        "- Whether model/output disclosures must be added for specific jurisdictions or platforms.",
        "- Whether additional consent is needed for crash reports, local storage, runtime downloads, or sensitive-content workflows.",
        "- Whether server-side deletion and export tooling must be implemented before release."
      ]
    },
    {
      "heading": "32. Legal-Review Disclaimer",
      "paragraphs": [
        "This document is an engineering and documentation aid. It is not legal advice and does not guarantee compliance with GDPR, UK GDPR, CCPA/CPRA, US state privacy laws, consumer-protection laws, payment-provider rules, platform policies, copyright law, AI regulations, or open-source license obligations.",
        "The operator must obtain review by qualified legal counsel before release and whenever the application, providers, data flows, licensing, telemetry, support practices, or business model materially change."
      ]
    }
  ],
  "notices": [
    {
      "heading": "Third-Party Notices",
      "paragraphs": [
        "Last updated: May 25, 2026",
        "This document summarizes third-party software, libraries, tools, APIs, models, and services identified from the inspected repository and repomix-output.xml. It is not a complete open-source license audit. Exact license texts, binary redistribution obligations, model licenses, and service terms must be verified by a qualified lawyer before commercial release.",
        "Product naming note: the repository and desktop bundle identify the product as \"AI YouTube Shorts Generator,\" while parts of the UI refer to \"Signal Forge.\" [VERIFY: final production product name]"
      ]
    },
    {
      "heading": "1. General Notice",
      "paragraphs": [
        "The application may include, depend on, execute, or connect to third-party software, services, APIs, models, datasets, platforms, and tools. Each third-party component remains governed by its own license, notices, terms, policies, and restrictions.",
        "Nothing in this application grants users rights to third-party content, websites, APIs, services, models, datasets, media, music, images, fonts, videos, software, or platform data beyond rights granted by the applicable owner or license.",
        "Users and distributors are responsible for complying with third-party terms, especially when downloading platform content, using API keys, redistributing binaries, bundling FFmpeg/runtime packs, downloading models, or publishing generated outputs."
      ]
    },
    {
      "heading": "2. Online Services and APIs",
      "paragraphs": [
        "The inspected codebase may use or connect to the following services.",
        "MuAPI: used in API mode for hosted processing, including youtube-download for source video download by URL and format, openai-whisper for hosted transcription, gpt-5-mini for transcript classification and highlight-selection prompts, autocrop for clip rendering/reframing, and polling endpoints under /predictions/{request_id}/result. The app sends MuAPI API keys in x-api-key headers and sends processing payloads such as source URLs, hosted media URLs, transcript-derived prompts, language settings, timestamps, and aspect ratios. License/terms verification required before release: MuAPI terms, data-processing terms, retention, content policy, pricing/rate limits, and attribution requirements.",
        "OpenAI: used by local mode through the Python openai package for highlight ranking and prompt-based text generation. The inspected Python default model is gpt-4o-mini unless overridden. The app may send transcript samples, timestamped transcript text, prompt instructions, and highlight-generation context to OpenAI. License/terms verification required before release: OpenAI API terms, privacy/data-use terms, model behavior disclaimers, output-use terms, and any required user disclosures.",
        "Gumroad: used by the Cloudflare Worker licensing backend for purchase/license verification. The Worker receives Gumroad webhook form data and verifies sales through the Gumroad sales API using an operator-configured Gumroad access token. Data may include sale IDs, product IDs, purchaser email, license key returned by Gumroad, refund/dispute signals, and provider sale metadata. License/terms verification required before release: Gumroad developer/API terms, webhook terms, payment/refund terms, and privacy/data-processing obligations.",
        "Cloudflare Workers and D1: used for the hosted licensing backend, including activation, validation, reset requests, reset status, admin review, Gumroad webhook handling, idempotency records, audit events, license records, and device bindings. License/terms verification required before release: Cloudflare terms, D1 data location/retention/security terms, Workers terms, and data-processing agreement availability.",
        "YouTube, Google, and other source platforms: the application can process YouTube URLs and local mode can use yt-dlp to download online video sources. API mode can send source URLs to MuAPI for hosted downloading. The application is not affiliated with or endorsed by YouTube, Google, TikTok, Instagram, Meta, or other platforms unless separately stated in writing. Users must comply with platform terms, copyright rules, downloader restrictions, API terms, and content policies.",
        "Update, runtime-pack, model-hosting, and crash-report endpoints: the inspected app includes Tauri updater integration, local runtime-pack download/repair flows, local model downloads through faster-whisper tooling, and optional user-submitted crash reports if a crash-report endpoint is configured. Release verification required: final update endpoint, updater signing/public key, runtime-pack manifest/asset host, runtime archive license notices, model-hosting provider terms, crash-report provider terms, and retention policy."
      ]
    },
    {
      "heading": "3. Frontend and Desktop UI Dependencies",
      "paragraphs": [
        "The app/package.json manifest identifies the following frontend and desktop JavaScript dependencies:",
        "- @tauri-apps/api",
        "- @tauri-apps/plugin-updater",
        "Development and test dependencies identified:",
        "- @tauri-apps/cli",
        "- @sveltejs/vite-plugin-svelte",
        "- svelte",
        "- vite",
        "- vitest",
        "- jsdom",
        "- @testing-library/svelte",
        "- @testing-library/jest-dom",
        "License metadata for these packages was not included in the inspected repomix file. Generate and review a final pnpm license report before release, including transitive dependencies from pnpm-lock.yaml if present in the release repository."
      ]
    },
    {
      "heading": "4. Rust, Tauri, and Native Dependencies",
      "paragraphs": [
        "The app/src-tauri/Cargo.toml manifest identifies these Rust/Tauri dependencies:",
        "- tauri",
        "- tauri-build",
        "- tauri-plugin-updater",
        "- serde",
        "- serde_json",
        "- reqwest with rustls-tls",
        "- tokio",
        "- rfd",
        "- sha2",
        "- async-trait",
        "- license-control-suite as a local path dependency under vendor/license-control-suite",
        "License metadata and transitive crate license data were not included in the inspected repomix file. Generate and review a final Cargo license report before release, including the local license-control-suite path dependency and any bundled license notices required by Tauri, Rust crates, and updater artifacts."
      ]
    },
    {
      "heading": "5. Python Dependencies",
      "paragraphs": [
        "The Python manifests identify:",
        "- requests",
        "- python-dotenv",
        "- yt-dlp",
        "- faster-whisper",
        "- openai",
        "- opencv-python",
        "- Optional torch for CUDA Whisper use",
        "Runtime validation and bundling scripts also reference or imply transitive/local-model dependencies such as:",
        "- ctranslate2",
        "- huggingface_hub",
        "- tokenizers",
        "- av",
        "- numpy",
        "License metadata for Python packages and transitive dependencies was not included in the inspected repomix file. Generate and review a final Python license report for both normal and optional local-mode dependencies before release."
      ]
    },
    {
      "heading": "6. Media Tools",
      "paragraphs": [
        "FFmpeg: the local pipeline invokes FFmpeg for cutting clips, encoding video with libx264, encoding audio with AAC, muxing audio, and producing MP4 outputs. FFmpeg is a third-party project governed by its own licensing. FFmpeg obligations can vary depending on whether FFmpeg is bundled, downloaded as a runtime pack, or required from the user's system; whether the build is LGPL, GPL, or includes non-free/commercial codec options; whether libx264, AAC, hardware encoders, or other codecs are enabled; and distribution platform and jurisdiction. Release verification required: exact FFmpeg binary source, version, build flags, enabled codecs, license mode, source-offer obligations, attribution, and whether FFmpeg is redistributed with the app or runtime pack.",
        "yt-dlp: the local pipeline uses yt-dlp to download source media by URL. The inspected bundled-runtime folder includes a yt-dlp launcher/wrapper and runtime information referencing yt-dlp. yt-dlp is third-party software with its own license and terms. Use of yt-dlp may also be restricted by the terms of the websites being accessed. Users are responsible for complying with platform terms and rights restrictions. Release verification required: exact yt-dlp version, license notice, whether a binary/wrapper is redistributed, update policy, and platform-compliance disclosures.",
        "Python runtime and runtime packs: the app can use a bundled Python runtime, a downloaded local runtime pack, or development/runtime paths. Runtime packs may include Python, site-packages, yt-dlp, FFmpeg, and bridge code depending on release packaging. Release verification required: Python license, bundled package licenses, runtime-pack archive notices, checksum/signature policy, model/cache license terms, and operating-system-specific packaging notices."
      ]
    },
    {
      "heading": "7. AI and Model Dependencies",
      "paragraphs": [
        "The application can use faster-whisper for local transcription and supports selectable Whisper model names such as tiny, base, small, medium, large-v3, large-v3-turbo, and English-specific variants.",
        "Model files may be downloaded and cached locally through faster-whisper or related model-hosting tooling. The inspected code stores local model cache files under an app-specific Hugging Face-style cache directory.",
        "Release verification required: exact model source, model license, model card terms, data-use restrictions, attribution requirements, redistribution rights, commercial-use restrictions, and whether model files are bundled or downloaded on demand.",
        "The local reframing implementation uses OpenCV Haar cascade functionality through opencv-python; verify OpenCV package and bundled data-file notices before release."
      ]
    },
    {
      "heading": "8. Backend and Worker Dependencies",
      "paragraphs": [
        "The worker/package.json manifest identifies a private Worker package without runtime npm dependencies in the inspected file. The Worker is intended to run on Cloudflare Workers and use Cloudflare D1.",
        "The Worker uses platform-provided Web APIs and Cloudflare bindings for request handling, crypto, D1 database access, and fetch calls to Gumroad.",
        "Development/deployment tooling such as Wrangler is referenced in scripts and documentation but not listed as a committed Worker dependency lockfile in the inspected manifest. Verify any development/deployment tooling licenses separately if redistributed or included in release materials."
      ]
    },
    {
      "heading": "9. Payment and Licensing Components",
      "paragraphs": [
        "The application uses a local Rust path dependency named license-control-suite for licensing/authentication contracts, desktop persistence, Tauri command wrappers, and Worker client behavior.",
        "The codebase also includes custom Worker logic for Gumroad sale verification, access-token issuance, hashed license-key records, device bindings, reset requests, admin review, idempotency records, and audit events.",
        "Release verification required: license terms for license-control-suite, Gumroad terms, Cloudflare terms, token/signing compliance, data-processing agreements, and refund/consumer-law obligations."
      ]
    },
    {
      "heading": "10. Testing and Development Dependencies",
      "paragraphs": [
        "The repository contains tests and scripts that reference Node test runner and Vitest, Testing Library for Svelte, jsdom, Cargo tests and Rust test utilities, Python characterization/parity tests, and shell scripts for secure pnpm install, Cargo checks, bundled runtime preparation, and supply-chain triage.",
        "These tools may not be distributed with production builds, but their licenses should be reviewed if included in source releases, binary bundles, CI artifacts, or developer distributions."
      ]
    },
    {
      "heading": "11. License Metadata Not Fully Discoverable From Repomix",
      "paragraphs": [
        "The inspected repomix file did not include complete lockfiles, transitive dependency inventories, full third-party license texts, FFmpeg build metadata, model cards, runtime-pack manifests, or final production service contracts.",
        "Before release, complete all of the following:",
        "- Generate an npm license report for frontend dependencies and transitive packages.",
        "- Generate a Cargo license report for Rust crates and the local path dependency.",
        "- Generate a Python license report for base and local-mode dependencies.",
        "- Verify FFmpeg, yt-dlp, Python runtime, and runtime-pack redistribution obligations.",
        "- Verify faster-whisper and model-file licenses for all selectable models.",
        "- Verify MuAPI, OpenAI, Gumroad, Cloudflare, update host, crash-report provider, model-host, and platform terms.",
        "- Include required license texts, copyright notices, attribution, source offers, and model notices in release artifacts.",
        "- Confirm whether the app's own license permits the planned distribution model."
      ]
    },
    {
      "heading": "12. User and Distributor Responsibility",
      "paragraphs": [
        "Users must comply with applicable third-party terms when using APIs, downloading content, processing platform media, publishing clips, or using generated outputs commercially.",
        "Distributors must verify and satisfy all third-party license obligations before shipping source code, binaries, installers, runtime packs, model files, or bundled media tools."
      ]
    }
  ],
  "refund": [
    {
      "heading": "Refund Policy",
      "paragraphs": [
        "Refund requests are handled manually within 7 days from purchase, subject to purchase records and platform dispute rules.",
        "No automated refund engine is built into this app.",
        "Refund eligibility may depend on the payment platform, purchase record, licensing status, dispute status, abuse prevention, and applicable law. [VERIFY: final refund terms before public release]"
      ]
    },
    {
      "heading": "Licensing and Access",
      "paragraphs": [
        "The app may require license activation, validation, device binding, session validation, and reset requests before protected features are available.",
        "A license may not work if it is invalid, revoked, expired, already bound to another device, blocked, reset-pending, affected by payment issues, or rejected by the licensing backend.",
        "The developers are not responsible for access interruptions caused by invalid purchases, chargebacks, Gumroad issues, license server downtime, network failures, device reset delays, device changes, local storage corruption, or unsupported environments."
      ]
    }
  ]
};

export const POLICY_COMMON_SECTIONS: PolicySection[] = [];

export const POLICY_LAST_UPDATED_LABEL = 'May 23, 2026';
