// Custom hook for Ed25519 key generation and management

import { useState, useCallback } from 'react';
import type {
  KeyGenerationState,
  KeyGenerationResult,
  SecurityApiResponse,
  KeyRegistrationRequest,
} from '../types/cryptography';
import { generateKeyPairWithBase64 } from '../utils/ed25519';
import { registerPublicKey as registerPublicKeyApi } from '../api/securityClient';

const INITIAL_RESULT: KeyGenerationResult = {
  keyPair: null,
  publicKeyBase64: null,
  error: null,
  isGenerating: false,
};

export function useKeyGeneration(): KeyGenerationState {
  const [result, setResult] = useState<KeyGenerationResult>(INITIAL_RESULT);

  const generateKeyPair = useCallback(async () => {
    setResult(prev => ({ ...prev, isGenerating: true, error: null }));
    
    try {
      const { keyPair, publicKeyBase64 } = await generateKeyPairWithBase64();

      setResult({
        keyPair,
        publicKeyBase64,
        error: null,
        isGenerating: false,
      });
    } catch (error) {
      setResult({
        keyPair: null,
        publicKeyBase64: null,
        error: error instanceof Error ? error.message : 'Failed to generate keypair',
        isGenerating: false,
      });
    }
  }, []);

  const clearKeys = useCallback(() => {
    setResult(INITIAL_RESULT);
  }, []);

  const registerPublicKey = useCallback(async (publicKeyBase64: string): Promise<boolean> => {
    try {
      const requestBody: KeyRegistrationRequest = {
        public_key: publicKeyBase64,
        owner_id: 'web-user', // Default owner ID for web interface
        permissions: ['read', 'write'], // Default permissions
        metadata: {
          generated_by: 'web-interface',
          generation_time: new Date().toISOString(),
          key_type: 'ed25519'
        },
        expires_at: null // No expiration by default
      };

      const data: SecurityApiResponse = await registerPublicKeyApi(requestBody);
      return data.success ?? false;
    } catch (error) {
      console.error('Failed to register public key:', error);
      return false;
    }
  }, []);

  return {
    result,
    generateKeyPair,
    clearKeys,
    registerPublicKey,
  };
}