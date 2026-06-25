import { describe, expect, it } from 'vitest';

// Canonical inventory of Admin UI sections, their decisions, and minimum auth policies
interface AdminSectionPolicy {
  decision: 'retain' | 'remove' | 'replace_with_devolens_link' | 'replace_with_devolens_flow';
  requiredAuthLevel: 'admin_token' | 'super_admin';
  rationale: string;
}

const ADMIN_SECTION_POLICIES: Record<string, AdminSectionPolicy> = {
  overview: {
    decision: 'retain',
    requiredAuthLevel: 'admin_token',
    rationale: 'Provides high-level metrics of D1 database and worker activity. Essential for monitoring.'
  },
  delete_requests: {
    decision: 'retain',
    requiredAuthLevel: 'super_admin',
    rationale: 'GDPR/privacy data erasure is worker-specific but must also block the corresponding Devolens key.'
  },
  audit_events: {
    decision: 'retain',
    requiredAuthLevel: 'super_admin',
    rationale: 'Compliance audit log of admin activity; must remain securely stored in D1.'
  },
  idempotency: {
    decision: 'retain',
    requiredAuthLevel: 'super_admin',
    rationale: 'Worker-level request replay protection cache; not a Devolens concern.'
  }
};

describe('Admin UI & Command Inventory', () => {
  const ACTUAL_SECTIONS = ['overview', 'delete_requests', 'audit_events', 'idempotency'];
  const REMOVED_SECTIONS = ['licenses', 'device_bindings', 'reset_requests'];

  it('asserts every active admin section has a documented Devolens-alignment decision', () => {
    for (const section of ACTUAL_SECTIONS) {
      const policy = ADMIN_SECTION_POLICIES[section];
      expect(policy, `Section "${section}" must be documented with an alignment decision.`).toBeDefined();
      expect(['retain', 'remove', 'replace_with_devolens_link', 'replace_with_devolens_flow']).toContain(policy.decision);
    }
  });

  it('asserts every active admin section requires a secure authorization level', () => {
    for (const section of ACTUAL_SECTIONS) {
      const policy = ADMIN_SECTION_POLICIES[section];
      expect(policy.requiredAuthLevel, `Section "${section}" must specify a required auth level.`).toBeDefined();
      expect(['admin_token', 'super_admin']).toContain(policy.requiredAuthLevel);
    }
  });

  it('enforces that deprecated reset requests are not active admin sections', () => {
    expect(ACTUAL_SECTIONS).not.toContain('reset_requests');
    expect(REMOVED_SECTIONS).toContain('reset_requests');
  });

  it('removes broad license and device browsing sections from the admin UI', () => {
    expect(ACTUAL_SECTIONS).not.toContain('licenses');
    expect(ACTUAL_SECTIONS).not.toContain('device_bindings');
    expect(REMOVED_SECTIONS).toContain('licenses');
    expect(REMOVED_SECTIONS).toContain('device_bindings');
  });
});
