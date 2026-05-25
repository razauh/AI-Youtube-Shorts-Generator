import type { AdminCommandError } from './contracts';

const FRIENDLY_MESSAGES: Record<string, string> = {
  unauthorized: 'Admin token is invalid or expired.',
  bad_request: 'The request was invalid.',
  reset_request_not_found: 'This reset request no longer exists.',
  invalid_transition: 'This request is no longer pending.',
  storage: 'The Worker storage service is temporarily unavailable.',
  network: 'Could not reach the Worker API.',
  serialization: 'Something went wrong. Check logs for details.',
  unknown: 'Something went wrong. Check logs for details.'
};

export function friendlyAdminError(error: unknown): AdminCommandError {
  if (error && typeof error === 'object') {
    const candidate = error as Partial<AdminCommandError>;
    const code = typeof candidate.code === 'string' ? candidate.code : 'unknown';
    return {
      code,
      message: FRIENDLY_MESSAGES[code] ?? FRIENDLY_MESSAGES.unknown,
      request_id: typeof candidate.request_id === 'string' ? candidate.request_id : null,
      retryable: Boolean(candidate.retryable)
    };
  }
  return { code: 'unknown', message: FRIENDLY_MESSAGES.unknown, request_id: null, retryable: false };
}

export function validateAdminConfig(baseUrl: string, token: string): string | null {
  const normalized = baseUrl.trim();
  if (!normalized) return 'Worker API base URL is required.';
  let parsed: URL;
  try {
    parsed = new URL(normalized);
  } catch {
    return 'Worker API base URL must be a valid http or https URL.';
  }
  if (!['http:', 'https:'].includes(parsed.protocol) || !parsed.host) {
    return 'Worker API base URL must be a valid http or https URL.';
  }
  if (!token.trim()) return 'Admin API token is required.';
  return null;
}

export function redactTokenForDisplay(token: string): string {
  const trimmed = token.trim();
  if (trimmed.length <= 4) return '[redacted]';
  return `[redacted]...${trimmed.slice(-4)}`;
}
