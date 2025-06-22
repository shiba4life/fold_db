import type { SignedMessage } from '../types/cryptography';
import type {
  ApiResponse,
  VerificationResponse,
  KeyRegistrationResponse,
} from '../types/api';
import type { KeyRegistrationRequest } from '../types/cryptography';

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

async function get<T>(endpoint: string): Promise<ApiResponse<T>> {
  try {
    const response = await fetch(`${API_BASE_URL}${endpoint}`, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
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

export async function registerPublicKey(
  request: KeyRegistrationRequest,
): Promise<ApiResponse<KeyRegistrationResponse>> {
  return post<KeyRegistrationResponse>('/system-key', request);
}

export async function getSystemPublicKey(): Promise<ApiResponse<{ public_key: string; public_key_id?: string }>> {
  return get('/system-key');
}
