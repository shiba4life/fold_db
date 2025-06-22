// Ed25519 utility functions
import * as ed from '@noble/ed25519';

/**
 * Convert a Uint8Array to a base64 string
 * @param {Uint8Array} bytes - The bytes to convert
 * @returns {string} The base64 encoded string
 */
export const bytesToBase64 = (bytes) => {
  return btoa(String.fromCharCode(...bytes));
};

/**
 * Convert a base64 string to a Uint8Array
 * @param {string} base64 - The base64 string to convert
 * @returns {Uint8Array} The decoded bytes
 */
export const base64ToBytes = (base64) => {
  return Uint8Array.from(atob(base64), c => c.charCodeAt(0));
};

/**
 * Convert a hex string to a Uint8Array
 * @param {string} hex - The hex string to convert
 * @returns {Uint8Array} The decoded bytes
 */
export const hexToBytes = (hex) => {
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < hex.length; i += 2) {
    bytes[i / 2] = parseInt(hex.substr(i, 2), 16);
  }
  return bytes;
};

/**
 * Convert a Uint8Array to a hex string
 * @param {Uint8Array} bytes - The bytes to convert
 * @returns {string} The hex encoded string
 */
export const bytesToHex = (bytes) => {
  return Array.from(bytes)
    .map(b => b.toString(16).padStart(2, '0'))
    .join('');
};

/**
 * Sign a message using Ed25519
 * @param {Uint8Array} message - The message to sign
 * @param {Uint8Array} privateKey - The private key to sign with
 * @returns {Promise<Uint8Array>} The signature bytes
 */
export const sign = async (message, privateKey) => {
  return await ed.signAsync(message, privateKey);
};

/**
 * Generate an Ed25519 keypair with base64 encoded public key
 * @returns {Promise<{keyPair: {privateKey: Uint8Array, publicKey: Uint8Array}, publicKeyBase64: string}>}
 */
export const generateKeyPairWithBase64 = async () => {
  try {
    // Generate a secure random private key
    const privateKey = ed.utils.randomPrivateKey();
    
    // Derive the public key from the private key
    const publicKey = await ed.getPublicKeyAsync(privateKey);
    
    const keyPair = {
      privateKey,
      publicKey
    };
    
    return {
      keyPair,
      publicKeyBase64: bytesToBase64(publicKey)
    };
  } catch (error) {
    throw new Error(`Failed to generate Ed25519 keypair: ${error.message}`);
  }
};