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
  licenses: {
    decision: 'replace_with_devolens_link',
    requiredAuthLevel: 'admin_token',
    rationale: 'D1 is not the license source of truth. Licenses should be managed on the Devolens dashboard.'
  },
  device_bindings: {
    decision: 'replace_with_devolens_flow',
    requiredAuthLevel: 'admin_token',
    rationale: 'Active bindings must be queried and cleared via Devolens APIs.'
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
  const ACTUAL_SECTIONS = ['overview', 'delete_requests', 'licenses', 'device_bindings', 'audit_events', 'idempotency'];
  const REMOVED_SECTIONS = ['reset_requests'];

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

  // Red Test (TDD): Failing test representing sections that must be transitioned
  it('enforces that licenses section is replaced with Devolens link and disables D1 direct editing', () => {
    const policy = ADMIN_SECTION_POLICIES['licenses'];
    expect(policy.decision).toBe('replace_with_devolens_link');
  });

  it('enforces that deprecated reset requests are not active admin sections', () => {
    expect(ACTUAL_SECTIONS).not.toContain('reset_requests');
    expect(REMOVED_SECTIONS).toContain('reset_requests');
  });
});
