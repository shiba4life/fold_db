import { useState, useCallback } from 'react';
import { useSigning } from './useSigning';
import { verifyMessage } from '../api/securityClient';
import type { KeyPair } from '../types/cryptography';
import type { VerificationResult } from '../types/api';

interface SecurityApiState {
  isLoading: boolean;
  error: string | null;
  result: VerificationResult | null;
}

export function useSecurityAPI(keyPair: KeyPair | null, publicKeyId: string) {
  const { signPayload, isSigning } = useSigning();
  const [state, setState] = useState<SecurityApiState>({
    isLoading: false,
    error: null,
    result: null,
  });

  const testSignatureVerification = useCallback(async () => {
    if (!keyPair) {
      setState({
        isLoading: false,
        error: 'Key pair is not available.',
        result: null,
      });
      return;
    }

    setState({ isLoading: true, error: null, result: null });

    const payload = {
      action: 'test-signature',
      timestamp: new Date().toISOString(),
    };

    const signedMessage = await signPayload(payload, publicKeyId, keyPair.privateKey);

    if (signedMessage) {
      const response = await verifyMessage(signedMessage);
      if (response.success && response.verification_result) {
        setState({
          isLoading: false,
          error: null,
          result: response.verification_result,
        });
      } else {
        setState({
          isLoading: false,
          error: response.error || 'Failed to verify message.',
          result: null,
        });
      }
    } else {
      setState({
        isLoading: false,
        error: 'Failed to sign the message.',
        result: null,
      });
    }
  }, [keyPair, publicKeyId, signPayload]);

  return {
    ...state,
    isSigning,
    testSignatureVerification,
  };
} 