/**
 * Ed25519 key pair interface
 */
export interface Ed25519KeyPair {
  /** The private key as Uint8Array (32 bytes) */
  privateKey: Uint8Array;
  /** The public key as Uint8Array (32 bytes) */
  publicKey: Uint8Array;
}

/**
 * Key generation options
 */
export interface KeyGenerationOptions {
  /** Whether to validate the generated keys (default: true) */
  validate?: boolean;
  /** Custom entropy source (for testing only) */
  entropy?: Uint8Array;
}

/**
 * Key format options for serialization
 */
export interface KeyFormatOptions {
  /** Output format: 'hex', 'base64', or 'uint8array' */
  format: 'hex' | 'base64' | 'uint8array';
}

/**
 * Error types for key generation
 */
export class Ed25519KeyError extends Error {
  constructor(message: string, public readonly code: string) {
    super(message);
    this.name = 'Ed25519KeyError';
  }
}

/**
 * Browser compatibility information
 */
export interface BrowserCompatibility {
  /** Whether the browser supports WebCrypto API */
  webCrypto: boolean;
  /** Whether the browser supports secure random generation */
  secureRandom: boolean;
  /** Whether the browser supports Ed25519 natively */
  nativeEd25519: boolean;
  /** Browser name and version information */
  browserInfo: string;
}

/**
 * Storage-related error types
 */
export class StorageError extends Error {
  constructor(message: string, public readonly code: string) {
    super(message);
    this.name = 'StorageError';
  }
}

/**
 * Metadata for stored keys
 */
export interface StoredKeyMetadata {
  /** Human-readable name for the key */
  name: string;
  /** Optional description */
  description: string;
  /** ISO timestamp when key was created */
  created: string;
  /** ISO timestamp when key was last accessed */
  lastAccessed: string;
  /** Optional tags for key organization */
  tags: string[];
}

/**
 * Storage configuration options
 */
export interface StorageOptions {
  /** Database name (optional, defaults to 'DataFoldKeyStorage') */
  dbName?: string;
  /** Enable debugging logs */
  debug?: boolean;
}

/**
 * Interface for key storage implementations
 */
export interface KeyStorageInterface {
  /** Store an encrypted key pair */
  storeKeyPair(
    keyId: string,
    keyPair: Ed25519KeyPair,
    passphrase: string,
    metadata?: Partial<StoredKeyMetadata>
  ): Promise<void>;

  /** Retrieve and decrypt a key pair */
  retrieveKeyPair(keyId: string, passphrase: string): Promise<Ed25519KeyPair>;

  /** Delete a stored key pair */
  deleteKeyPair(keyId: string): Promise<void>;

  /** List all stored key metadata */
  listKeys(): Promise<Array<{ id: string; metadata: StoredKeyMetadata }>>;

  /** Check if a key exists */
  keyExists(keyId: string): Promise<boolean>;

  /** Get storage usage information */
  getStorageInfo(): Promise<{ used: number; available: number | null }>;

  /** Clear all stored keys */
  clearAllKeys(): Promise<void>;

  /** Close storage connection */
  close(): Promise<void>;
}

/**
 * Key derivation configuration options
 */
export interface KeyDerivationOptions {
  /** Derivation algorithm: 'HKDF' or 'PBKDF2' */
  algorithm: 'HKDF' | 'PBKDF2';
  /** Number of iterations for PBKDF2 (ignored for HKDF) */
  iterations?: number;
  /** Hash function for derivation */
  hash?: 'SHA-256' | 'SHA-384' | 'SHA-512';
  /** Salt for derivation (optional, random if not provided) */
  salt?: Uint8Array;
  /** Info parameter for HKDF (required for HKDF) */
  info?: Uint8Array;
  /** Length of derived key in bytes (default: 32) */
  length?: number;
}

/**
 * Derived key information
 */
export interface DerivedKeyInfo {
  /** The derived key material */
  key: Uint8Array;
  /** Algorithm used for derivation */
  algorithm: 'HKDF' | 'PBKDF2';
  /** Salt used for derivation */
  salt: Uint8Array;
  /** Info parameter (for HKDF) */
  info?: Uint8Array;
  /** Number of iterations (for PBKDF2) */
  iterations?: number;
  /** Hash function used */
  hash: string;
  /** Timestamp when key was derived */
  derived: string;
}

/**
 * Key rotation configuration
 */
export interface KeyRotationOptions {
  /** Whether to keep old key version for backward compatibility */
  keepOldVersion?: boolean;
  /** Custom metadata for the rotated key */
  metadata?: Partial<StoredKeyMetadata>;
  /** Reason for rotation */
  reason?: string;
  /** Whether to update derived keys as well */
  rotateDerivedKeys?: boolean;
}

/**
 * Key version information for rotation tracking
 */
export interface KeyVersion {
  /** Version number */
  version: number;
  /** Key pair for this version */
  keyPair: Ed25519KeyPair;
  /** Timestamp when this version was created */
  created: string;
  /** Whether this version is active */
  active: boolean;
  /** Reason for this version (e.g., 'initial', 'rotation', 'recovery') */
  reason: string;
  /** Derived keys for this version */
  derivedKeys?: Record<string, DerivedKeyInfo>;
}

/**
 * Versioned key pair with rotation support
 */
export interface VersionedKeyPair {
  /** Key identifier */
  keyId: string;
  /** Current active version number */
  currentVersion: number;
  /** All versions of this key */
  versions: Record<number, KeyVersion>;
  /** Metadata for the key pair */
  metadata: StoredKeyMetadata;
}

/**
 * Key rotation result
 */
export interface KeyRotationResult {
  /** The new key pair */
  newKeyPair: Ed25519KeyPair;
  /** The new version number */
  newVersion: number;
  /** The previous version number */
  previousVersion: number;
  /** Whether old version was preserved */
  oldVersionPreserved: boolean;
  /** List of rotated derived keys */
  rotatedDerivedKeys: string[];
}

/**
 * Error types for key derivation and rotation
 */
export class KeyDerivationError extends Error {
  constructor(message: string, public readonly code: string) {
    super(message);
    this.name = 'KeyDerivationError';
  }
}

export class KeyRotationError extends Error {
  constructor(message: string, public readonly code: string) {
    super(message);
    this.name = 'KeyRotationError';
  }
}

/**
 * Export/Import error types
 */
export class KeyExportError extends Error {
  constructor(message: string, public readonly code: string) {
    super(message);
    this.name = 'KeyExportError';
  }
}

export class KeyImportError extends Error {
  constructor(message: string, public readonly code: string) {
    super(message);
    this.name = 'KeyImportError';
  }
}

/**
 * Export format options
 */
export type ExportFormat = 'json' | 'binary';

/**
 * Export options for key backup
 */
export interface KeyExportOptions {
  /** Export format: 'json' or 'binary' */
  format: ExportFormat;
  /** Include metadata in export */
  includeMetadata?: boolean;
  /** Custom KDF iterations (default: 100000) */
  kdfIterations?: number;
  /** Additional data to include in export */
  additionalData?: Record<string, any>;
}

/**
 * Import options for key restoration
 */
export interface KeyImportOptions {
  /** Whether to validate key integrity after import */
  validateIntegrity?: boolean;
  /** Whether to overwrite existing keys with same ID */
  overwriteExisting?: boolean;
  /** Custom metadata to merge with imported metadata */
  customMetadata?: Partial<StoredKeyMetadata>;
}

/**
 * Encrypted backup format (JSON)
 */
export interface EncryptedBackupFormat {
  /** Format version for future compatibility */
  version: number;
  /** Key derivation function used */
  kdf: 'pbkdf2';
  /** KDF parameters */
  kdf_params: {
    /** Base64-encoded salt */
    salt: string;
    /** Number of iterations */
    iterations: number;
    /** Hash function used */
    hash: string;
  };
  /** Encryption algorithm used */
  encryption: 'aes-gcm';
  /** Base64-encoded nonce/IV */
  nonce: string;
  /** Base64-encoded encrypted data */
  ciphertext: string;
  /** ISO timestamp when backup was created */
  created: string;
  /** Optional additional authenticated data */
  aad?: string;
  /** Backup format type */
  type: 'datafold-key-backup';
}

/**
 * Key backup data structure (before encryption)
 */
export interface KeyBackupData {
  /** Key identifier */
  keyId: string;
  /** Base64-encoded private key */
  privateKey: string;
  /** Base64-encoded public key */
  publicKey: string;
  /** Key metadata */
  metadata: StoredKeyMetadata;
  /** Backup creation timestamp */
  exported: string;
  /** SDK version that created the backup */
  sdkVersion: string;
}

/**
 * Export result information
 */
export interface KeyExportResult {
  /** The exported data (JSON string or Uint8Array) */
  data: string | Uint8Array;
  /** Export format used */
  format: ExportFormat;
  /** Size of exported data in bytes */
  size: number;
  /** Checksum for integrity verification */
  checksum: string;
  /** Export timestamp */
  timestamp: string;
}

/**
 * Import result information
 */
export interface KeyImportResult {
  /** Imported key identifier */
  keyId: string;
  /** Whether the key was overwritten */
  overwritten: boolean;
  /** Imported key metadata */
  metadata: StoredKeyMetadata;
  /** Import timestamp */
  timestamp: string;
  /** Whether integrity validation passed */
  integrityValid: boolean;
}

/**
 * Import validation result
 */
export interface ImportValidationResult {
  /** Whether the import data is valid */
  valid: boolean;
  /** Validation issues found */
  issues: string[];
  /** Detected format */
  format?: ExportFormat;
  /** Backup version if detected */
  version?: number;
}

/**
 * Server connection configuration
 */
export interface ServerConnectionConfig {
  /** Base URL of the DataFold server */
  baseUrl: string;
  /** Request timeout in milliseconds */
  timeout: number;
  /** Number of retries for failed requests */
  retries: number;
  /** Base delay between retries in milliseconds */
  retryDelay: number;
  /** Retry configuration */
  retryConfig?: RetryConfig;
}

/**
 * Retry configuration for HTTP requests
 */
export interface RetryConfig {
  /** Maximum number of retries */
  maxRetries: number;
  /** Base delay between retries in milliseconds */
  baseDelay: number;
  /** Maximum delay between retries in milliseconds */
  maxDelay: number;
  /** Backoff multiplier for exponential backoff */
  backoffFactor: number;
}

/**
 * Error types for server communication
 */
export class DataFoldServerError extends Error {
  constructor(
    message: string,
    public readonly errorCode: string,
    public readonly httpStatus: number = 0,
    public readonly details: Record<string, any> = {}
  ) {
    super(message);
    this.name = 'DataFoldServerError';
  }
}

/**
 * Public key registration request
 */
export interface PublicKeyRegistrationRequest {
  /** Optional client identifier (generated if not provided) */
  clientId?: string;
  /** Optional user identifier */
  userId?: string;
  /** Hex-encoded Ed25519 public key */
  publicKey: string;
  /** Optional human-readable key name */
  keyName?: string;
  /** Optional metadata */
  metadata?: Record<string, string>;
}

/**
 * Public key registration response
 */
export interface PublicKeyRegistrationResponse {
  /** Unique registration identifier */
  registrationId: string;
  /** Client identifier */
  clientId: string;
  /** Hex-encoded public key */
  publicKey: string;
  /** Key name if provided */
  keyName?: string;
  /** Registration timestamp */
  registeredAt: string;
  /** Registration status */
  status: string;
}

/**
 * Public key status response
 */
export interface PublicKeyStatusResponse {
  /** Registration identifier */
  registrationId: string;
  /** Client identifier */
  clientId: string;
  /** Hex-encoded public key */
  publicKey: string;
  /** Key name if provided */
  keyName?: string;
  /** Registration timestamp */
  registeredAt: string;
  /** Current status */
  status: string;
  /** Last used timestamp */
  lastUsed?: string;
}

/**
 * Signature verification request
 */
export interface SignatureVerificationRequest {
  /** Client identifier */
  clientId: string;
  /** Message to verify */
  message: string;
  /** Hex-encoded signature */
  signature: string;
  /** Message encoding format */
  messageEncoding?: 'utf8' | 'hex' | 'base64';
  /** Optional metadata */
  metadata?: Record<string, string>;
}

/**
 * Signature verification response
 */
export interface SignatureVerificationResponse {
  /** Whether verification succeeded */
  verified: boolean;
  /** Client identifier */
  clientId: string;
  /** Hex-encoded public key used for verification */
  publicKey: string;
  /** Verification timestamp */
  verifiedAt: string;
  /** SHA-256 hash of the message */
  messageHash: string;
}

/**
 * Server integration client interface
 */
export interface ServerIntegrationInterface {
  /** Test connection to the server */
  testConnection(): Promise<{ connected: boolean; latency?: number; error?: string }>;
  
  /** Register a public key with the server */
  registerPublicKey(request: PublicKeyRegistrationRequest): Promise<PublicKeyRegistrationResponse>;
  
  /** Get public key registration status */
  getPublicKeyStatus(clientId: string): Promise<PublicKeyStatusResponse>;
  
  /** Verify a digital signature */
  verifySignature(request: SignatureVerificationRequest): Promise<SignatureVerificationResponse>;
}