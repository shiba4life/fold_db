/**
 * Utility functions for request signing
 */

import { SigningError, DigestAlgorithm, ContentDigest } from './types.js';

/**
 * Generate a UUID v4 nonce for replay protection
 */
export function generateNonce(): string {
  // Use crypto.randomUUID if available (modern browsers and Node.js 15.6+)
  if (typeof crypto !== 'undefined' && crypto.randomUUID) {
    return crypto.randomUUID();
  }

  // Fallback implementation using crypto.getRandomValues
  if (typeof crypto !== 'undefined' && crypto.getRandomValues) {
    const array = new Uint8Array(16);
    crypto.getRandomValues(array);
    
    // Set version (4) and variant bits according to UUID v4 spec
    array[6] = (array[6] & 0x0f) | 0x40; // Version 4
    array[8] = (array[8] & 0x3f) | 0x80; // Variant 10

    const hex = Array.from(array, byte => byte.toString(16).padStart(2, '0')).join('');
    return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20, 32)}`;
  }

  throw new SigningError(
    'Unable to generate secure nonce - crypto API not available',
    'CRYPTO_UNAVAILABLE'
  );
}

/**
 * Generate current Unix timestamp
 */
export function generateTimestamp(): number {
  return Math.floor(Date.now() / 1000);
}

/**
 * Validate nonce format (should be UUID v4)
 */
export function validateNonce(nonce: string): boolean {
  const uuidPattern = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
  return uuidPattern.test(nonce);
}

/**
 * Validate timestamp (should be reasonable Unix timestamp)
 */
export function validateTimestamp(timestamp: number): boolean {
  // Check if it's a valid Unix timestamp (after 2000 and before 2100)
  const year2000 = 946684800; // 2000-01-01 00:00:00 UTC
  const year2100 = 4102444800; // 2100-01-01 00:00:00 UTC
  
  return Number.isInteger(timestamp) && timestamp >= year2000 && timestamp <= year2100;
}

/**
 * Parse URL to extract components needed for signing
 */
export function parseUrl(url: string): { origin: string; pathname: string; search: string; targetUri: string } {
  try {
    const urlObj = new URL(url);
    const targetUri = urlObj.pathname + urlObj.search;
    
    return {
      origin: urlObj.origin,
      pathname: urlObj.pathname,
      search: urlObj.search,
      targetUri
    };
  } catch (error) {
    throw new SigningError(
      `Invalid URL format: ${url}`,
      'INVALID_URL',
      { originalError: error instanceof Error ? error.message : 'Unknown error' }
    );
  }
}

/**
 * Normalize header name to lowercase for consistent processing
 */
export function normalizeHeaderName(name: string): string {
  return name.toLowerCase().trim();
}

/**
 * Validate header name for inclusion in signature
 */
export function validateHeaderName(name: string): boolean {
  // RFC 7230 compliant header name validation
  const headerNamePattern = /^[!#$%&'*+\-.0-9A-Z^_`a-z|~]+$/;
  return headerNamePattern.test(name);
}

/**
 * Calculate content digest for request body
 */
export async function calculateContentDigest(
  content: string | Uint8Array,
  algorithm: DigestAlgorithm = 'sha-256'
): Promise<ContentDigest> {
  if (typeof crypto === 'undefined' || !crypto.subtle) {
    throw new SigningError(
      'Web Crypto API not available for digest calculation',
      'CRYPTO_UNAVAILABLE'
    );
  }

  try {
    // Convert content to Uint8Array if needed
    const data = typeof content === 'string' 
      ? new TextEncoder().encode(content)
      : content;

    // Map algorithm names to Web Crypto API names
    const algoMap: Record<DigestAlgorithm, string> = {
      'sha-256': 'SHA-256',
      'sha-512': 'SHA-512'
    };

    const webCryptoAlgo = algoMap[algorithm];
    if (!webCryptoAlgo) {
      throw new SigningError(
        `Unsupported digest algorithm: ${algorithm}`,
        'UNSUPPORTED_ALGORITHM'
      );
    }

    // Calculate hash
    const hashBuffer = await crypto.subtle.digest(webCryptoAlgo, data);
    const hashArray = new Uint8Array(hashBuffer);
    
    // Convert to base64
    const base64 = btoa(String.fromCharCode(...hashArray));
    
    // Format header value according to RFC 3230
    const headerValue = `${algorithm}=:${base64}:`;

    return {
      algorithm,
      digest: base64,
      headerValue
    };
  } catch (error) {
    throw new SigningError(
      `Failed to calculate content digest: ${error instanceof Error ? error.message : 'Unknown error'}`,
      'DIGEST_CALCULATION_FAILED',
      { algorithm, originalError: error instanceof Error ? error.message : 'Unknown error' }
    );
  }
}

/**
 * Escape string value for use in signature parameters
 */
export function escapeParameterValue(value: string): string {
  // Escape quotes and backslashes
  return value.replace(/\\/g, '\\\\').replace(/"/g, '\\"');
}

/**
 * Quote parameter value if it contains special characters
 */
export function quoteParameterValue(value: string): string {
  // Quote if value contains special characters
  if (/[\s";=,]/.test(value)) {
    return `"${escapeParameterValue(value)}"`;
  }
  return value;
}

/**
 * Format Unix timestamp as RFC 3339 string (for logging/debugging)
 */
export function formatTimestamp(timestamp: number): string {
  return new Date(timestamp * 1000).toISOString();
}

/**
 * Secure memory clearing utility (best effort in JavaScript)
 */
export function clearSensitiveData(data: Uint8Array): void {
  if (data && typeof crypto !== 'undefined' && crypto.getRandomValues) {
    // Overwrite with random data first, then zeros
    crypto.getRandomValues(data);
    data.fill(0);
  }
}

/**
 * Validate Ed25519 private key format for signing
 */
export function validateSigningPrivateKey(privateKey: Uint8Array): boolean {
  return privateKey instanceof Uint8Array && privateKey.length === 32;
}

/**
 * Validate Ed25519 public key format for signing
 */
export function validateSigningPublicKey(publicKey: Uint8Array): boolean {
  return publicKey instanceof Uint8Array && publicKey.length === 32;
}

/**
 * Convert Uint8Array to hex string
 */
export function toHex(bytes: Uint8Array): string {
  return Array.from(bytes, byte => byte.toString(16).padStart(2, '0')).join('');
}

/**
 * Convert hex string to Uint8Array
 */
export function fromHex(hex: string): Uint8Array {
  if (hex.length % 2 !== 0) {
    throw new SigningError('Invalid hex string length', 'INVALID_HEX');
  }
  
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < hex.length; i += 2) {
    const byte = parseInt(hex.substr(i, 2), 16);
    if (isNaN(byte)) {
      throw new SigningError('Invalid hex character', 'INVALID_HEX');
    }
    bytes[i / 2] = byte;
  }
  
  return bytes;
}

/**
 * Performance timing utility
 */
export class PerformanceTimer {
  private startTime: number;

  constructor() {
    this.startTime = performance.now();
  }

  elapsed(): number {
    return performance.now() - this.startTime;
  }

  reset(): void {
    this.startTime = performance.now();
  }
}