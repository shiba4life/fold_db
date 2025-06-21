import { useState, useCallback } from 'react';
import { createSignedMessage } from '../utils/signing';
import type { SignedMessage } from '../types/cryptography';

interface UseSigningState {
  isSigning: boolean;
  error: Error | null;
}

export function useSigning() {
  const [signingState, setSigningState] = useState<UseSigningState>({
    isSigning: false,
    error: null,
  });

  const signPayload = useCallback(
    async (
      payload: any,
      publicKeyId: string,
      privateKey: Uint8Array
    ): Promise<SignedMessage | null> => {
      setSigningState({ isSigning: true, error: null });
      try {
        const signedMessage = await createSignedMessage(
          payload,
          publicKeyId,
          privateKey
        );
        setSigningState({ isSigning: false, error: null });
        return signedMessage;
      } catch (error) {
        const err = error instanceof Error ? error : new Error('Failed to sign payload');
        setSigningState({ isSigning: false, error: err });
        return null;
      }
    },
    []
  );

  return {
    ...signingState,
    signPayload,
  };
} 