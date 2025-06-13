/**
 * TypeScript type definitions for PBI-12 Key Rotation protocol
 * These types match the backend API for server-coordinated key rotation
 */

import { Ed25519KeyPair } from '../types.js';

/**
 * Key rotation reasons supported by the server
 */
export type ServerRotationReason = 
  | 'compromise'
  | 'scheduled' 
  | 'policy'
  | 'emergency'
  | 'manual'
  | 'expiration';

/**
 * Key rotation request for server API
 */
export interface ServerKeyRotationRequest {
  /** Old public key (hex encoded) */
  old_public_key: string;
  /** New public key (hex encoded) */
  new_public_key: string;
  /** Rotation reason */
  reason: ServerRotationReason;
  /** Request timestamp (ISO string) */
  timestamp: string;
  /** Request signature using old private key */
  signature: string;
  /** Optional metadata */
  metadata?: Record<string, string>;
}

/**
 * Key rotation response from server API
 */
export interface ServerKeyRotationResponse {
  /** Whether the rotation was successful */
  success: boolean;
  /** New key identifier/fingerprint */
  new_key_id: string;
  /** Confirmation that old key was invalidated */
  old_key_invalidated: boolean;
  /** Operation correlation ID for audit trail */
  correlation_id: string;
  /** Response timestamp */
  timestamp: string;
  /** Any warnings or notes */
  warnings: string[];
  /** Number of associations updated */
  associations_updated: number;
}

/**
 * Key rotation status response
 */
export interface ServerKeyRotationStatusResponse {
  /** Operation correlation ID */
  correlation_id: string;
  /** Current status of the rotation */
  status: string;
  /** Old public key (hex encoded) */
  old_public_key: string;
  /** New public key (hex encoded) */
  new_public_key: string;
  /** Rotation reason */
  reason: ServerRotationReason;
  /** When the operation started */
  started_at: string;
  /** When the operation completed (if applicable) */
  completed_at?: string;
  /** Number of associations updated */
  associations_updated: number;
  /** Error details (if failed) */
  error_details?: string;
}

/**
 * Key rotation history entry
 */
export interface ServerKeyRotationHistoryEntry {
  /** Operation correlation ID */
  correlation_id: string;
  /** Operation status */
  status: string;
  /** Old public key (hex encoded) */
  old_public_key: string;
  /** New public key (hex encoded) */
  new_public_key: string;
  /** Rotation reason */
  reason: ServerRotationReason;
  /** When the operation started */
  started_at: string;
  /** When the operation completed */
  completed_at?: string;
  /** Actor who initiated the rotation */
  actor?: string;
}

/**
 * Key rotation validation result
 */
export interface ServerKeyRotationValidationResult {
  /** Whether the request is valid */
  valid: boolean;
  /** Validation errors */
  errors: string[];
  /** Validation warnings */
  warnings: string[];
  /** Request ID */
  request_id: string;
}

/**
 * Extended key rotation options for server coordination
 */
export interface ServerKeyRotationOptions {
  /** Whether to keep old version for backward compatibility */
  keepOldVersion?: boolean;
  /** Custom metadata for the rotated key */
  metadata?: Record<string, string>;
  /** Rotation reason */
  reason: ServerRotationReason;
  /** Whether to update derived keys as well */
  rotateDerivedKeys?: boolean;
  /** Force rotation even with warnings */
  force?: boolean;
  /** Actor identifier */
  actor?: string;
  /** Timeout for server operation (ms) */
  timeout?: number;
  /** Whether to verify backup before rotation */
  verifyBackup?: boolean;
  /** Progress callback for real-time updates */
  onProgress?: (progress: RotationProgress) => void;
}

/**
 * Key rotation progress information
 */
export interface RotationProgress {
  /** Current step in the rotation process */
  step: RotationStep;
  /** Progress percentage (0-100) */
  percentage: number;
  /** Human-readable status message */
  message: string;
  /** Optional details about current operation */
  details?: Record<string, any>;
  /** Whether this step completed successfully */
  completed: boolean;
  /** Any errors encountered */
  error?: string;
}

/**
 * Steps in the key rotation process
 */
export type RotationStep = 
  | 'validating_request'
  | 'verifying_backup'
  | 'generating_new_key'
  | 'signing_request'
  | 'submitting_to_server'
  | 'waiting_for_confirmation'
  | 'updating_local_storage'
  | 'rotating_derived_keys'
  | 'finalizing'
  | 'completed'
  | 'failed';

/**
 * Key backup verification result
 */
export interface BackupVerificationResult {
  /** Whether backup verification passed */
  verified: boolean;
  /** Issues found during verification */
  issues: string[];
  /** Backup format detected */
  format?: string;
  /** Backup creation timestamp */
  created?: string;
  /** Whether backup can recover the key */
  recoverable: boolean;
}

/**
 * Key rotation recovery options
 */
export interface RotationRecoveryOptions {
  /** Correlation ID of failed rotation */
  correlationId: string;
  /** Whether to rollback to previous key */
  rollback?: boolean;
  /** Whether to retry the rotation */
  retry?: boolean;
  /** Custom recovery metadata */
  metadata?: Record<string, string>;
}

/**
 * Key rotation recovery result
 */
export interface RotationRecoveryResult {
  /** Whether recovery was successful */
  success: boolean;
  /** Recovery action taken */
  action: 'rollback' | 'retry' | 'manual_intervention_required';
  /** Current key state after recovery */
  currentKeyState: {
    keyId: string;
    version: number;
    active: boolean;
  };
  /** Recovery details */
  details: Record<string, any>;
}

/**
 * Atomic rotation state for client-side tracking
 */
export interface AtomicRotationState {
  /** Unique operation ID */
  operationId: string;
  /** Current phase of the atomic operation */
  phase: 'preparing' | 'submitted' | 'confirmed' | 'completed' | 'failed' | 'rolled_back';
  /** Original key pair being rotated */
  oldKeyPair: Ed25519KeyPair;
  /** New key pair for rotation */
  newKeyPair: Ed25519KeyPair;
  /** Server correlation ID */
  serverCorrelationId?: string;
  /** Backup created for this rotation */
  backupData?: string;
  /** Rotation options used */
  options: ServerKeyRotationOptions;
  /** Progress information */
  progress: RotationProgress[];
  /** When the operation started */
  startedAt: string;
  /** When the operation completed */
  completedAt?: string;
  /** Any error that occurred */
  error?: string;
}

/**
 * Security evaluation result for rotation requests
 */
export interface SecurityEvaluationResult {
  /** Whether the operation is allowed */
  allowed: boolean;
  /** Calculated risk score (0.0 to 1.0) */
  risk_score: number;
  /** Security warnings (non-blocking) */
  warnings: string[];
  /** Security violations (blocking) */
  violations: string[];
  /** Required actions to take */
  required_actions: string[];
}

/**
 * Error types for server key rotation
 */
export class ServerKeyRotationError extends Error {
  constructor(
    message: string,
    public readonly code: string,
    public readonly correlationId?: string,
    public readonly serverResponse?: any
  ) {
    super(message);
    this.name = 'ServerKeyRotationError';
  }
}

export class BackupVerificationError extends Error {
  constructor(
    message: string,
    public readonly code: string,
    public readonly issues: string[] = []
  ) {
    super(message);
    this.name = 'BackupVerificationError';
  }
}

export class RotationRecoveryError extends Error {
  constructor(
    message: string,
    public readonly code: string,
    public readonly correlationId?: string
  ) {
    super(message);
    this.name = 'RotationRecoveryError';
  }
}

export class SecurityPolicyViolationError extends Error {
  constructor(
    message: string,
    public readonly code: string,
    public readonly violations: string[] = [],
    public readonly riskScore?: number
  ) {
    super(message);
    this.name = 'SecurityPolicyViolationError';
  }
}