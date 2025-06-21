// Ed25519 cryptographic utilities using @noble/ed25519

import * as ed from '@noble/ed25519';
import type { KeyPair } from '../types/cryptography';

/**
 * Generate a new Ed25519 keypair
 * @returns Promise<KeyPair> - Generated keypair
 */
export async function generateEd25519KeyPair(): Promise<KeyPair> {
  try {
    // Generate a secure random private key
    const privateKey = ed.utils.randomPrivateKey();
    
    // Derive the public key from the private key
    const publicKey = await ed.getPublicKeyAsync(privateKey);
    
    return {
      privateKey,
      publicKey
    };
  } catch (error) {
    throw new Error(`Failed to generate Ed25519 keypair: ${error instanceof Error ? error.message : 'Unknown error'}`);
  }
}

/**
 * Convert a Uint8Array to hex string
 * @param bytes - Uint8Array to convert
 * @returns hex string
 */
export function bytesToHex(bytes: Uint8Array): string {
  return Array.from(bytes)
    .map(b => b.toString(16).padStart(2, '0'))
    .join('');
}

/**
 * Convert hex string to Uint8Array
 * @param hex - hex string to convert
 * @returns Uint8Array
 */
export function hexToBytes(hex: string): Uint8Array {
  if (hex.length % 2 !== 0) {
    throw new Error('Hex string must have even length');
  }
  
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < hex.length; i += 2) {
    bytes[i / 2] = parseInt(hex.substr(i, 2), 16);
  }
  return bytes;
}

/**
 * Validate if a string is a valid hex representation
 * @param hex - string to validate
 * @returns boolean
 */
export function isValidHex(hex: string): boolean {
  return /^[0-9a-fA-F]*$/.test(hex) && hex.length % 2 === 0;
}

/**
 * Generate a keypair and return both raw bytes and hex representations
 */
export async function generateKeyPairWithHex(): Promise<{
  keyPair: KeyPair;
  publicKeyHex: string;
  privateKeyHex: string;
}> {
  const keyPair = await generateEd25519KeyPair();
  
  return {
    keyPair,
    publicKeyHex: bytesToHex(keyPair.publicKey),
    privateKeyHex: bytesToHex(keyPair.privateKey)
  };
}