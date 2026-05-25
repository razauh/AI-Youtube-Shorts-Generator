import { describe, expect, it } from 'vitest';
import { friendlyAdminError, redactTokenForDisplay, validateAdminConfig } from '../../admin/lib/messages';

describe('admin messages and validation', () => {
  it('maps unauthorized errors without exposing token material', () => {
    const error = friendlyAdminError({
      code: 'unauthorized',
      message: 'raw worker message',
      request_id: 'req_123',
      retryable: false
    });

    expect(error.message).toBe('Admin token is invalid or expired.');
    expect(error.request_id).toBe('req_123');
    expect(JSON.stringify(error)).not.toContain('admin-secret');
  });

  it('maps invalid transition errors to pending-state language', () => {
    const error = friendlyAdminError({ code: 'invalid_transition', request_id: 'req_456' });
    expect(error.message).toBe('This request is no longer pending.');
  });

  it('validates base URL and token input', () => {
    expect(validateAdminConfig('', 'token')).toBe('Worker API base URL is required.');
    expect(validateAdminConfig('file:///tmp/token', 'token')).toBe('Worker API base URL must be a valid http or https URL.');
    expect(validateAdminConfig('https://worker.example.test', '')).toBe('Admin API token is required.');
    expect(validateAdminConfig('https://worker.example.test', 'token')).toBeNull();
  });

  it('redacts token display to the last four characters only', () => {
    const redacted = redactTokenForDisplay('admin-token-secret-1234');
    expect(redacted).toBe('[redacted]...1234');
    expect(redacted).not.toContain('admin-token-secret');
  });
});
