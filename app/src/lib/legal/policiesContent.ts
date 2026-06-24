export type PolicyTab = 'terms' | 'privacy' | 'deletion' | 'compliance' | 'notices' | 'refund';

type PolicySection = {
  heading: string;
  paragraphs: string[];
};

export const POLICY_LAST_UPDATED_LABEL = 'May 30, 2026';

export const POLICY_SECTIONS: Record<PolicyTab, PolicySection[]> = {
  terms: [
    {
      heading: 'Terms and Conditions',
      paragraphs: [
        'Last updated: May 30, 2026',
        'These Terms and Conditions apply to your use of AI YouTube Shorts Generator.',
        'The application is a desktop tool for generating short-form video clips from user-provided YouTube URLs using API-based processing.'
      ]
    },
    {
      heading: '1. Acceptance of Terms',
      paragraphs: [
        'By installing, activating, accessing, or using the application, you agree to these Terms. If you do not agree, do not use the application.',
        'If you use the application on behalf of a company or organization, you represent that you have authority to accept these Terms on its behalf.'
      ]
    },
    {
      heading: '2. Description of the Application',
      paragraphs: [
        'The application may submit source URLs and processing settings to MuAPI-hosted AI and media-processing endpoints for download, transcription, highlight processing, autocrop, and rendered clip output.',
        'The application is a tool. It does not guarantee perfect results, lawful outputs, platform acceptance, monetization, engagement, ranking, visibility, or business results.',
        'You are responsible for reviewing generated video, transcripts, titles, hooks, metadata, scores, and exported JSON before use.'
      ]
    },
    {
      heading: '3. User Responsibilities',
      paragraphs: [
        'You are responsible for your use of the application and for all content you upload, import, process, generate, export, publish, or distribute.',
        'You must ensure you have rights and permissions for source media and generated outputs and comply with copyright, licensing, privacy, publicity rights, platform rules, API provider terms, and applicable law.',
        'The application does not provide legal, copyright, platform-policy, business, or professional advice.'
      ]
    },
    {
      heading: '4. Third-Party APIs and Services',
      paragraphs: [
        'The application may use or connect to Devolens (Cryptolens), MuAPI, Gumroad, Cloudflare Workers and D1, YouTube, Google, source platforms, update hosts, and crash-report endpoints if configured.',
        'Third-party APIs and services are controlled by their respective providers. We do not control their availability, pricing, rate limits, account policies, content policies, output quality, security practices, privacy practices, or terms.',
        'You are responsible for complying with all provider terms, policies, usage limits, billing requirements, and content rules.'
      ]
    },
    {
      heading: '5. API Mode and MuAPI Processing',
      paragraphs: [
        'In API mode, the application may send processing inputs to MuAPI. These inputs may include source URLs, media references, transcript-related data, highlight data, prompt-like processing data, timing data, aspect-ratio settings, and other information needed to generate clips.',
        'MuAPI may perform download, transcription, highlight processing, LLM-based ranking, autocrop, and media-rendering tasks. MuAPI is a third-party service, and its own terms and policies apply.',
        'The application\'s supported generation workflow depends on MuAPI availability. If MuAPI, its API, or its hosted processing services stop working, become unavailable, change, degrade, rate-limit, suspend access, or discontinue support, that outage or service change is outside our control and is not our responsibility.',
        'We are not responsible for MuAPI downtime, processing failures, incorrect results, hosted output availability, rate limits, account restrictions, billing, policy changes, or media-processing errors.'
      ]
    },
    {
      heading: '6. AI Processing',
      paragraphs: [
        'The application may use hosted AI processing through configured API-mode providers for transcription, title generation, hook generation, scoring, classification, or similar tasks.',
        'AI-generated outputs may be inaccurate, incomplete, biased, offensive, misleading, unexpected, duplicative, low quality, legally risky, or unsuitable for your intended use.',
        'You must verify all AI-generated outputs before relying on them or publishing them.'
      ]
    },
    {
      heading: '7. FFmpeg and Media Processing',
      paragraphs: [
        'MuAPI or other hosted providers may use FFmpeg or similar media-processing tools for cutting, encoding, muxing, converting, reframing, and generating media files.',
        'FFmpeg is a third-party project governed by its own licensing. Distributors remain responsible for any notices required by components they ship.',
        'You are responsible for checking all generated media files before publishing or distributing them.'
      ]
    },
    {
      heading: '8. User Content and Intellectual Property',
      paragraphs: [
        'You retain whatever rights you already have in your own content. The application does not grant you rights to third-party videos, music, images, likenesses, voices, transcripts, datasets, fonts, APIs, software, or other protected material.',
        'You must only process content that you own or have permission to use.',
        'We are not responsible for copyright claims, DMCA notices, takedowns, rejected uploads, demonetization, lost revenue, account strikes, account bans, platform enforcement, legal claims, or disputes caused by your source content, generated outputs, uploads, or publications.'
      ]
    },
    {
      heading: '9. Diagnostics, Paths, and Settings',
      paragraphs: [
        'The application may provide diagnostics, API key profile settings, output path controls, crash draft handling, and advanced app information.',
        'Some settings and paths affect whether the application works correctly. You are responsible for changes to output paths, working directories, permissions, configuration files, or diagnostics values.',
        'Logs, crash drafts, diagnostics, output JSON, error messages, and support materials may include sensitive or identifying information. You should review them before sharing.'
      ]
    },
    {
      heading: '10. Privacy and Data Handling',
      paragraphs: [
        'The application supports API-based processing. Local project history, settings, output metadata, exported JSON, crash drafts, reset status, generated outputs, logs, configuration files, and license/session/device-related information may be stored on your device.',
        'External transmission may occur when you use MuAPI/API mode, activate or validate a license, request a device reset, the licensing worker verifies a Gumroad purchase, check for or install updates, or submit a crash report to a configured endpoint.',
        'No general telemetry or analytics SDK was identified during repository inspection.'
      ]
    },
    {
      heading: '11. Licensing, Activation, and Device Binding',
      paragraphs: [
        'The application may require license activation and validation using Devolens (Cryptolens) licensing services. Licensing may include license keys, device binding, session validation, reset requests, Gumroad purchase verification, purchaser email records, server-side license records, and local license/session/device state.',
        'A license may not work if it is invalid, revoked, expired, already bound to another device, blocked, reset-pending, affected by payment issues, or rejected by the licensing backend.',
        'Refund eligibility is handled through the payment provider or support channel and may depend on purchase status, licensing status, dispute status, abuse prevention, and applicable consumer law.'
      ]
    },
    {
      heading: '12. Third-Party Licenses',
      paragraphs: [
        'Third-party libraries, APIs, services, tools, and other components remain governed by their own licenses and terms.',
        'These may include Devolens (Cryptolens), MuAPI, Gumroad, Cloudflare, YouTube, Google, FFmpeg, Rust crates, Node/pnpm packages, Tauri components, Svelte/Vite tooling, and license-control-suite.',
        'Distributors remain responsible for including exact notices for any binary, package, service contract, or tool they ship.'
      ]
    },
    {
      heading: '13. Security and Credentials',
      paragraphs: [
        'API key values, license/session tokens, admin tokens where configured, and device key material are intended to be stored using operating-system credential storage when available.',
        'On Windows and macOS, license/session fallback storage uses a protected local encryption key and encrypted app-data fallback files for session tokens and device key material. Raw license keys are not intended to be stored in fallback files.',
        'Linux secure persistence support has known limitations in the initial release configuration. Linux fallback files are not used for plaintext license/session secret persistence, and Linux secure-storage limitations are planned to be addressed in future updates without a specific date or guarantee.'
      ]
    },
    {
      heading: '14. Updates and Changes',
      paragraphs: [
        'The application may be updated over time. Updates may add, remove, or change features, APIs, supported providers, output formats, diagnostics behavior, settings, licensing behavior, supported operating systems, security behavior, or system requirements.',
        'We do not guarantee that any specific feature, provider, workflow, or output format will remain available.',
        'These Terms may also be updated over time. Continued use of the application after updated Terms become effective means you accept the updated Terms.'
      ]
    },
    {
      heading: '15. Termination',
      paragraphs: [
        'We may suspend or terminate access where permitted by law if a license is invalid, revoked, refunded, charged back, disputed, abused, or used in violation of these Terms.',
        'Refunded, charged-back, revoked, disabled, or disputed purchases may lose access.',
        'You may stop using and uninstall the application at any time.'
      ]
    },
    {
      heading: '16. Prohibited Uses',
      paragraphs: [
        'You must not use the application to violate any law or regulation; infringe copyright, trademark, privacy, publicity, or other rights; process private, confidential, or sensitive content without permission; create spam, deceptive content, or fraudulent content; manipulate platforms, rankings, engagement, recommendations, or monetization systems; scrape, download, or process content without authorization; violate API provider terms or platform terms; misuse API keys, credentials, license keys, or accounts; or bypass, tamper with, disable, or interfere with licensing, activation, device binding, security controls, or access controls except where applicable law expressly permits.'
      ]
    },
    {
      heading: '17. No Warranty',
      paragraphs: [
        'The application is provided "as is" and "as available." To the maximum extent permitted by law, we disclaim all warranties, whether express, implied, statutory, or otherwise.',
        'We do not guarantee that the application will be error-free, uninterrupted, secure, compatible with your environment, accepted by any platform, monetized, ranked, visible, profitable, or suitable for your intended purpose.',
        'We do not guarantee output correctness, transcript accuracy, highlight quality, title quality, hook quality, media quality, API availability, diagnostics accuracy, or update availability.'
      ]
    },
    {
      heading: '18. Limitation of Liability',
      paragraphs: [
        'To the maximum extent permitted by law, we are not liable for indirect, incidental, special, consequential, exemplary, punitive, or similar damages.',
        'We are not liable for losses or claims involving data loss, lost revenue, lost profits, account bans, account strikes, API costs, unexpected charges, publishing mistakes, copyright claims, takedowns, rejected uploads, failed uploads, demonetization, corrupted outputs, damaged media files, failed conversions, AI mistakes, dependency failures, third-party service failures, license activation issues, device reset delays, unsupported environments, user-modified settings, user-modified paths, or disclosed diagnostic information.',
        'No monetary liability cap is included in this version unless separately agreed in writing; applicable consumer-law rights are not limited by these Terms.'
      ]
    }
  ],
  privacy: [
    {
      heading: 'Privacy Notice',
      paragraphs: [
        'Last updated: May 30, 2026',
        'This notice describes privacy-relevant behavior for AI YouTube Shorts Generator.',
        'The application stores project history, generated output metadata, settings, API key profile metadata, crash drafts, reset status, exported JSON, logs, and license/session/device-related information on your device where needed for app operation.'
      ]
    },
    {
      heading: 'Data You Provide',
      paragraphs: [
        'You may provide YouTube URLs, generation settings, project names, API keys, license keys, purchaser email for support flows, and deletion request details.',
        'MuAPI may receive source URLs, media references, transcript-related data, prompt-like processing data, timing data, and aspect-ratio settings for API-based processing.',
        'Gumroad, Devolens (Cryptolens), and Cloudflare Workers and D1 may process purchase, license, activation, reset, and deletion request data. This includes IP addresses, purchaser emails, license keys, and device machine codes needed for activation and license compliance.'
      ]
    },
    {
      heading: 'Storage and Transmission',
      paragraphs: [
        'Local app storage is used for project history, settings, crash drafts, and status caches. Secure storage is used where available for API key and session material.',
        'Crash reports are submitted only when an endpoint is configured and the user submits a draft.',
        'No general telemetry or analytics SDK was identified during repository inspection.'
      ]
    }
  ],
  deletion: [
    {
      heading: 'Data Deletion Notice',
      paragraphs: [
        'Last updated: May 30, 2026',
        'This notice explains the deletion request flow implemented in AI YouTube Shorts Generator. It is based on the app, Tauri command, admin desktop, Cloudflare Worker, and database behavior identified in the current repository.',
        'The in-app deletion request targets backend licensing data. When the application is operating in Devolens (Cryptolens) mode, the request directly blocks the license key in Devolens to anonymize/delete the associated licensing and device binding data. It does not automatically delete local app data on your device, generated clips, exported JSON, crash drafts, logs, MuAPI-held data, Gumroad purchase or payment records, update-host records, or support records held outside the licensing system.'
      ]
    },
    {
      heading: 'Who Can Submit a Request',
      paragraphs: [
        'A user can submit a backend licensing-data deletion request from Settings -> Policies -> Data Deletion by entering the license key, optionally entering the purchaser email, and typing DELETE.',
        'In Devolens mode, the license key is used to synchronously block and deactivate the license key in Devolens (Cryptolens). The request is designed for license-related backend data and does not require manual administrator review or approval.'
      ]
    },
    {
      heading: 'What The Request Covers',
      paragraphs: [
        'The deletion request blocks the license key and deactivates all associated device bindings in Devolens. This prevents future activations and validations of the license, effectively deleting the personal device binding records from the active licensing store.',
        'For legacy requests submitted during custom worker mode, requests are recorded for administrative review where an admin can review, approve, or reject the request.'
      ]
    },
    {
      heading: 'What The Request Does Not Cover',
      paragraphs: [
        'The backend deletion request does not remove local project history, onboarding state, theme preference, cached deletion request status, generated output metadata, exported JSON, locally saved clips or links, crash drafts, logs, configuration files, API key profiles, secure-store entries, or files the user created outside the Worker-backed licensing database.',
        'To remove local data, uninstalling the app may not be enough on every operating system. Users should also remove local app data, generated outputs, exported JSON files, logs, crash drafts, and any files they saved or shared if they want those local copies deleted.',
        'The request does not delete data controlled by third-party providers. MuAPI, Gumroad, Cloudflare, YouTube, Google, source platforms, update hosts, and support channels may have their own records, retention periods, legal obligations, deletion procedures, and privacy terms.'
      ]
    },
    {
      heading: 'Request Tracking and Lookup Token',
      paragraphs: [
        'After submission, the Worker returns a request ID, status, message, and lookup token. The app stores the request ID, status, and message in local app storage and attempts to store the lookup token through secure storage.',
        'The lookup token is required to refresh deletion status. If the lookup token is lost, unavailable, or invalid, the app cannot retrieve status through the implemented status endpoint without another valid token.',
        'The status endpoint accepts the request ID and lookup token and returns the current status, message, completion time if available, and safe error code if available.'
      ]
    },
    {
      heading: 'Admin Review and Processing',
      paragraphs: [
        'Deletion requests are not processed automatically. The admin desktop lists deletion requests by status, shows the masked license or license hash prefix, purchaser email when available, requested scope, preview counts, timestamps, details, and safe error information.',
        'An admin can reject a pending request with an optional reason. An admin can approve a pending or failed request only after typing DELETE USER DATA in the admin confirmation dialog.',
        'Approval records processing phases in Worker request metadata, changes status through approved and processing, performs deletion or anonymization actions, marks completed when successful, and records safe failure details if processing fails.'
      ]
    },
    {
      heading: 'Records That May Remain',
      paragraphs: [
        'Some records may remain after completion where needed for security, audit, fraud prevention, legal obligations, dispute handling, tax or payment records, support history, provider obligations, or to preserve a minimal record that the deletion request was handled.',
        'The implemented Worker completion flow keeps sanitized deletion request records and audit events. Audit metadata is intended to avoid raw license keys and raw purchaser email in completion events.',
        'Refunded, charged-back, revoked, disabled, disputed, or privacy-deleted purchases may lose access to the application because the approved deletion flow disables the backend license entitlement.'
      ]
    },
    {
      heading: 'Privacy Rights and Applicable Law',
      paragraphs: [
        'Depending on where the user lives and which law applies, users may have rights to request access, deletion or erasure, correction, portability, restriction, objection, opt-out, appeal, or to lodge a complaint with a regulator. Examples include GDPR-style erasure rights and US state privacy deletion rights.',
        'The app provides a concrete deletion request mechanism for backend licensing data, but the operator remains responsible for reviewing requests, deciding whether an exception or retention duty applies, responding through the appropriate support or purchase channel, and complying with applicable privacy law.',
        'The operator should not discriminate against users for exercising applicable privacy rights, except that deleting or anonymizing license records can technically prevent license validation, device binding, reset handling, support lookup, or continued access where those records are needed to provide the licensed app.'
      ]
    },
    {
      heading: 'How To Use This Notice',
      paragraphs: [
        'Use the in-app request form for backend licensing-data deletion. Use the request ID and lookup token to refresh status in the same Policies page.',
        'For local data, remove local app data and generated files from the device directly. For third-party records, use the relevant provider privacy process or support channel.',
        'For questions, disputes, appeals, or requests that the in-app flow cannot complete, use the Gumroad purchase or support channel associated with the license unless the operator has provided a separate privacy contact.'
      ]
    }
  ],
  compliance: [
    {
      heading: 'Data Compliance',
      paragraphs: [
        'Last updated: May 30, 2026',
        'The application includes user-initiated data deletion request flows for backend licensing data.',
        'The operator remains responsible for reviewing deletion requests, preserving legally required records, and complying with applicable privacy law.'
      ]
    },
    {
      heading: 'Security Controls',
      paragraphs: [
        'Security controls identified or expected include license-gated UI for generation features, device binding and signed access tokens, server-side hashed license keys with hash pepper, admin bearer-token authentication, masked emails/license keys in several UI/API responses, local secure-store use where available, crash-draft redaction for selected secret/license patterns, structured error mapping to avoid exposing raw auth failures, and no general telemetry/analytics SDK identified in inspected code.',
        'Before release, the operator should create an incident response plan covering suspected exposure of Worker secrets, Gumroad tokens, admin tokens, API keys, license keys, access tokens, or D1 data; unauthorized admin access; accidental support log or crash-report disclosure; user notification and regulatory notification timelines under applicable law; and token revocation, secret rotation, forensic preservation, and post-incident remediation.',
        'Data may move across borders through Devolens (Cryptolens), Cloudflare, Gumroad, MuAPI, update hosts, crash-report endpoints, and support operations.'
      ]
    }
  ],
  notices: [
    {
      heading: 'Third-Party Notices',
      paragraphs: [
        'Last updated: May 30, 2026',
        'The application uses or may interact with Devolens (Cryptolens), MuAPI, Gumroad, Cloudflare Workers and D1, YouTube, Google, source platforms, update hosts, FFmpeg, Rust, Tauri, and Native Dependencies, Vite, Tauri/Rust desktop app with Svelte UI, and license-control-suite.',
        'Exact license metadata is not shown in this in-app screen. The operator must generate release notices from committed manifests, lockfiles, transitive dependency inventories, full third-party license texts, final production service contracts, and distributed binary contents.'
      ]
    },
    {
      heading: 'Provider Terms',
      paragraphs: [
        'MuAPI: used for hosted video and AI/media processing. Verify API terms, privacy/data-use terms, billing terms, content policy, retention behavior, output ownership, and rate limits before release.',
        'YouTube, Google, and other source platforms: the application can process YouTube URLs through API-based hosted processing. The application is not affiliated with or endorsed by YouTube, Google, TikTok, Instagram, Meta, or other platforms unless separately stated in writing.'
      ]
    }
  ],
  refund: [
    {
      heading: 'Refund Policy',
      paragraphs: [
        'Last updated: May 30, 2026',
        'Refund requests are handled through the payment provider or support channel and may depend on purchase status, licensing status, dispute status, abuse prevention, and applicable consumer law.',
        'Refunded, charged-back, revoked, disabled, or disputed purchases may lose access.'
      ]
    },
    {
      heading: 'Support Boundaries',
      paragraphs: [
        'Support cannot guarantee platform acceptance, monetization, visibility, engagement, provider availability, or output quality.',
        'Users remain responsible for source-media rights, provider account standing, API usage costs, and publication decisions.',
        'The operator may request redacted diagnostics or crash drafts to investigate issues.'
      ]
    }
  ]
};

export const POLICY_COMMON_SECTIONS: PolicySection[] = [
  {
    heading: 'Common Notices',
    paragraphs: [
      'Use of the application remains subject to applicable law, provider terms, payment-provider rules, and platform policies.',
      'The application may be updated over time, and features may change or be removed.',
      'Last updated: May 30, 2026'
    ]
  }
];
