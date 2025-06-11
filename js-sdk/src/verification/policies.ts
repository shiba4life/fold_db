/**
 * Verification Policy Constants for JavaScript SDK
 */

import fs from 'fs';
import path from 'path';

export interface VerificationPolicy {
  name: string;
  description: string;
  verifyTimestamp: boolean;
  maxTimestampAge?: number;
  verifyNonce: boolean;
  verifyContentDigest: boolean;
  requiredComponents: string[];
  allowedAlgorithms: string[];
  requireAllHeaders: boolean;
}

// Load shared policies once
let _sharedPolicies: Record<string, VerificationPolicy> | null = null;

function getSharedPolicies(): Record<string, VerificationPolicy> {
  if (!_sharedPolicies) {
    const configPath = path.join(__dirname, '..', '..', '..', 'config', 'shared-policies.json');
    const configData = fs.readFileSync(configPath, 'utf8');
    const rawPolicies = JSON.parse(configData);
    
    // Convert raw policies to proper types
    _sharedPolicies = {};
    for (const [key, policy] of Object.entries(rawPolicies)) {
      _sharedPolicies[key] = {
        ...policy as any,
        maxTimestampAge: (policy as any).maxTimestampAge === null ? undefined : (policy as any).maxTimestampAge
      };
    }
  }
  return _sharedPolicies!;
}

// Export standard policy constants
const policies = getSharedPolicies();
export const STRICT: VerificationPolicy = policies.STRICT;
export const STANDARD: VerificationPolicy = policies.STANDARD;
export const LENIENT: VerificationPolicy = policies.LENIENT;
export const LEGACY: VerificationPolicy = policies.LEGACY;

// Export utility functions
export function getVerificationPolicy(name: string): VerificationPolicy | undefined {
  const policies = getSharedPolicies();
  return policies[name.toUpperCase()];
}

export function getAvailableVerificationPolicies(): string[] {
  const policies = getSharedPolicies();
  return Object.keys(policies);
}

export { policies as VERIFICATION_POLICIES };