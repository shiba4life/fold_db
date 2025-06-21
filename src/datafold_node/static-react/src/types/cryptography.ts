// Types for Ed25519 key management

export interface KeyPair {
  privateKey: Uint8Array;
  publicKey: Uint8Array;
}

export interface KeyGenerationResult {
  keyPair: KeyPair | null;
  publicKeyBase64: string | null;
  error: string | null;
  isGenerating: boolean;
}

export interface KeyGenerationState {
  result: KeyGenerationResult;
  generateKeyPair: () => Promise<void>;
  clearKeys: () => void;
  registerPublicKey: (publicKeyBase64: string) => Promise<boolean>;
}

export interface SecurityApiResponse {
  success: boolean;
  message?: string;
  data?: any;
}

export interface KeyRegistrationRequest {
  public_key: string;
  owner_id: string;
  permissions: string[];
  metadata: Record<string, any>;
  expires_at: number | null;
}

export interface SignedMessage {
  payload: any; // The original JSON payload
  signature: string; // base64 encoded
  public_key_id: string;
  timestamp: number; // UNIX timestamp in seconds
  nonce?: string;
}