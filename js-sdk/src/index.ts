/**
 * DataFold JavaScript SDK
 * Client-side key management with Ed25519 support
 */

// Export core types
export * from './types.js';

// Export Ed25519 functionality
export * from './crypto/ed25519.js';

// Export utilities
export * from './utils/validation.js';

// Export storage functionality
export * from './storage/index.js';

// Export key derivation functionality
export * from './crypto/key-derivation.js';

// Export key rotation functionality
export * from './crypto/key-rotation.js';

// Export key export/import functionality
export * from './crypto/key-export-import.js';

// Export server integration functionality
export * from './server/index.js';

// Export enhanced HTTP client functionality
export {
  createFluentHttpClient,
  createSignedHttpClient,
  createSigningMiddleware,
  createCorrelationMiddleware,
  createLoggingMiddleware,
  createPerformanceMiddleware
} from './server/http-client.js';

// Export request signing functionality (selective to avoid conflicts)
export {
  // Main signing classes and functions
  RFC9421Signer,
  createSigner,
  signRequest,
  
  // Configuration
  SigningConfigBuilder,
  createSigningConfig,
  createFromProfile,
  
  // Types for signing
  RFC9421SignatureResult,
  SigningConfig,
  SignableRequest,
  SigningOptions,
  SignatureComponents,
  SigningError,
  
  // Canonical message functions
  buildCanonicalMessage,
  buildSignatureInput,
  
  // Security profiles
  SECURITY_PROFILES,
  
  // Signing utilities (renamed to avoid conflicts)
  generateNonce,
  generateTimestamp,
  validateSigningPrivateKey,
  validateSigningPublicKey,
  calculateContentDigest
} from './signing/index.js';

// Export signature verification functionality
export {
  // Main verification classes and functions
  RFC9421Verifier,
  createVerifier,
  verifySignature as verifySignatureQuick,
  
  // Verification policies
  VERIFICATION_POLICIES,
  STRICT_VERIFICATION_POLICY,
  STANDARD_VERIFICATION_POLICY,
  LENIENT_VERIFICATION_POLICY,
  LEGACY_VERIFICATION_POLICY,
  createVerificationPolicy,
  getVerificationPolicy,
  
  // Inspector utilities
  RFC9421Inspector,
  createInspector,
  validateSignatureFormat,
  quickDiagnostic,
  
  // Middleware
  createResponseVerificationMiddleware,
  createRequestVerificationMiddleware,
  createExpressVerificationMiddleware,
  createFetchInterceptor,
  createBatchVerifier,
  
  // Verification types
  VerificationResult,
  VerificationConfig,
  VerificationPolicy,
  VerificationError,
  VerifiableResponse,
  ExtractedSignatureData
} from './verification/index.js';

// Export version information
export const SDK_VERSION = '0.1.0';

/**
 * SDK initialization and compatibility check
 */
export async function initializeSDK(): Promise<{ compatible: boolean; warnings: string[] }> {
  const warnings: string[] = [];
  let compatible = true;

  // Check browser compatibility
  const { checkBrowserCompatibility } = await import('./crypto/ed25519.js');
  const compat = checkBrowserCompatibility();

  if (!compat.webCrypto) {
    warnings.push('WebCrypto API not available - some features may not work');
    compatible = false;
  }

  if (!compat.secureRandom) {
    warnings.push('Secure random generation not available - key generation will fail');
    compatible = false;
  }

  if (!compat.nativeEd25519) {
    warnings.push('Native Ed25519 not supported - using JavaScript implementation');
    // This is not a compatibility issue, just informational
  }

  return { compatible, warnings };
}

/**
 * Quick compatibility check (synchronous)
 */
export function isCompatible(): boolean {
  return typeof crypto !== 'undefined' && 
         typeof crypto.subtle !== 'undefined' && 
         typeof crypto.getRandomValues !== 'undefined';
}