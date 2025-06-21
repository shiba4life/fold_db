import type { SignedMessage } from '../types/cryptography';
import type { ApiResponse, VerificationResponse } from '../types/api';

const API_BASE_URL = '/api/security';

async function post<T>(endpoint: string, body: any): Promise<ApiResponse<T>> {
  try {
    const response = await fetch(`${API_BASE_URL}${endpoint}`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(body),
    });

    if (!response.ok) {
      try {
        const errorData = await response.json();
        return {
          success: false,
          error: errorData.error || `HTTP error! status: ${response.status}`,
        };
      } catch (e) {
        return {
          success: false,
          error: `HTTP error! status: ${response.status}`,
        };
      }
    }
    
    // The backend sometimes returns success without a data field
    const responseData = await response.json();
    return {
      success: true,
      ...responseData,
    };

  } catch (error) {
    return {
      success: false,
      error: error instanceof Error ? error.message : 'An unknown network error occurred',
    };
  }
}

export async function verifyMessage(
  signedMessage: SignedMessage
): Promise<ApiResponse<VerificationResponse>> {
  return post<VerificationResponse>('/verify-message', signedMessage);
} 