export interface ApiResponse<T = any> {
  success: boolean;
  message?: string;
  error?: string;
  data?: T;
}

export interface VerificationResult {
  is_valid: boolean;
  timestamp_valid: boolean;
  owner_id?: string;
  permissions?: string[];
  error?: string;
}

export interface VerificationResponse {
  verification_result: VerificationResult;
}

export interface KeyRegistrationResponse {
  public_key_id?: string;
}
