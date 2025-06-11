/**
 * Test vectors and utilities for signature verification testing
 */

import {
  VerificationTestVector,
  VerifiableResponse
} from './types.js';
import { SignableRequest } from '../signing/types.js';
import { generateNonce, generateTimestamp, toHex } from '../signing/utils.js';

/**
 * Test key pair for verification testing
 */
export const TEST_KEY_PAIR = {
  // Ed25519 test private key (32 bytes)
  privateKey: new Uint8Array([
    0x9d, 0x61, 0xb1, 0x9d, 0xef, 0xfd, 0x5a, 0x60,
    0xba, 0x84, 0x4a, 0xf4, 0x92, 0xec, 0x2c, 0xc4,
    0x44, 0x49, 0xc5, 0x69, 0x7b, 0x32, 0x69, 0x19,
    0x70, 0x3b, 0xac, 0x03, 0x1c, 0xae, 0x7f, 0x60
  ]),
  
  // Ed25519 test public key (32 bytes)
  publicKey: new Uint8Array([
    0xd7, 0x5a, 0x98, 0x01, 0x82, 0xb1, 0x0a, 0xb7,
    0xd5, 0x4b, 0xfe, 0xd3, 0xc9, 0x64, 0x07, 0x3a,
    0x0e, 0xe1, 0x72, 0xf3, 0xda, 0xa6, 0x23, 0x25,
    0xaf, 0x02, 0x1a, 0x68, 0xf7, 0x07, 0x51, 0x1a
  ]),
  
  keyId: 'test-key-ed25519'
};

/**
 * Valid test vectors for positive verification tests
 */
export const VALID_TEST_VECTORS: VerificationTestVector[] = [
  {
    name: 'valid-basic-request',
    description: 'Basic valid signed GET request',
    category: 'positive',
    input: {
      message: {
        method: 'GET',
        url: 'https://api.example.com/data',
        headers: {
          'user-agent': 'DataFold-JS-SDK/0.1.0'
        }
      },
      headers: {
        'signature-input': 'sig1=("@method" "@target-uri");created=1640995200;keyid="test-key-ed25519";alg="ed25519";nonce="550e8400-e29b-41d4-a716-446655440000"',
        'signature': 'sig1=:d2pmTvmbncD3xQm8E9ZV2828BjQWGgiwAaw5bAkgibUopemLJcWDy/lkbbHAve7I2l1nirKIcOpxD4DBMq8hBg==:'
      },
      publicKey: TEST_KEY_PAIR.publicKey
    },
    expected: {
      status: 'valid',
      signatureValid: true,
      specificChecks: {
        formatValid: true,
        cryptographicValid: true
      }
    },
    metadata: {
      rfc9421Compliant: true,
      securityLevel: 'medium',
      tags: ['basic', 'get-request']
    }
  },

  {
    name: 'valid-post-with-content',
    description: 'Valid signed POST request with content digest',
    category: 'positive',
    input: {
      message: {
        method: 'POST',
        url: 'https://api.example.com/users',
        headers: {
          'content-type': 'application/json',
          'user-agent': 'DataFold-JS-SDK/0.1.0'
        },
        body: '{"name":"John Doe","email":"john@example.com"}'
      },
      headers: {
        'signature-input': 'sig1=("@method" "@target-uri" "content-type" "content-digest");created=1640995200;keyid="test-key-ed25519";alg="ed25519";nonce="550e8400-e29b-41d4-a716-446655440001"',
        'signature': 'sig1=:AbCdEfGhIjKlMnOpQrStUvWxYz1234567890abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ==:',
        'content-type': 'application/json',
        'content-digest': 'sha-256=:X48E9qOokqqrvdts8nOJRJN3OWDUoyWxBf7kbu9DBPE=:'
      },
      publicKey: TEST_KEY_PAIR.publicKey
    },
    expected: {
      status: 'valid',
      signatureValid: true,
      specificChecks: {
        formatValid: true,
        cryptographicValid: true,
        contentDigestValid: true
      }
    },
    metadata: {
      rfc9421Compliant: true,
      securityLevel: 'high',
      tags: ['post-request', 'content-digest', 'json']
    }
  }
];

/**
 * Invalid test vectors for negative verification tests
 */
export const INVALID_TEST_VECTORS: VerificationTestVector[] = [
  {
    name: 'invalid-missing-signature',
    description: 'Request missing signature header',
    category: 'negative',
    input: {
      message: {
        method: 'GET',
        url: 'https://api.example.com/data',
        headers: {}
      },
      headers: {
        'signature-input': 'sig1=("@method" "@target-uri");created=1640995200;keyid="test-key-ed25519";alg="ed25519";nonce="550e8400-e29b-41d4-a716-446655440000"'
        // Missing signature header
      },
      publicKey: TEST_KEY_PAIR.publicKey
    },
    expected: {
      status: 'invalid',
      signatureValid: false
    },
    metadata: {
      rfc9421Compliant: false,
      securityLevel: 'none',
      tags: ['missing-header', 'format-error']
    }
  },

  {
    name: 'invalid-malformed-signature',
    description: 'Request with malformed signature',
    category: 'negative',
    input: {
      message: {
        method: 'GET',
        url: 'https://api.example.com/data',
        headers: {}
      },
      headers: {
        'signature-input': 'sig1=("@method" "@target-uri");created=1640995200;keyid="test-key-ed25519";alg="ed25519";nonce="550e8400-e29b-41d4-a716-446655440000"',
        'signature': 'invalid-signature-format'
      },
      publicKey: TEST_KEY_PAIR.publicKey
    },
    expected: {
      status: 'invalid',
      signatureValid: false,
      specificChecks: {
        formatValid: false
      }
    },
    metadata: {
      rfc9421Compliant: false,
      securityLevel: 'none',
      tags: ['malformed', 'format-error']
    }
  },

  {
    name: 'invalid-wrong-signature',
    description: 'Request with cryptographically invalid signature',
    category: 'negative',
    input: {
      message: {
        method: 'GET',
        url: 'https://api.example.com/data',
        headers: {}
      },
      headers: {
        'signature-input': 'sig1=("@method" "@target-uri");created=1640995200;keyid="test-key-ed25519";alg="ed25519";nonce="550e8400-e29b-41d4-a716-446655440000"',
        'signature': 'sig1=:0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000:'
      },
      publicKey: TEST_KEY_PAIR.publicKey
    },
    expected: {
      status: 'invalid',
      signatureValid: false,
      specificChecks: {
        formatValid: true,
        cryptographicValid: false
      }
    },
    metadata: {
      rfc9421Compliant: true,
      securityLevel: 'none',
      tags: ['invalid-crypto', 'wrong-signature']
    }
  },

  {
    name: 'invalid-expired-timestamp',
    description: 'Request with expired timestamp',
    category: 'negative',
    input: {
      message: {
        method: 'GET',
        url: 'https://api.example.com/data',
        headers: {}
      },
      headers: {
        'signature-input': 'sig1=("@method" "@target-uri");created=1000000000;keyid="test-key-ed25519";alg="ed25519";nonce="550e8400-e29b-41d4-a716-446655440000"',
        'signature': 'sig1=:d2pmTvmbncD3xQm8E9ZV2828BjQWGgiwAaw5bAkgibUopemLJcWDy/lkbbHAve7I2l1nirKIcOpxD4DBMq8hBg==:'
      },
      publicKey: TEST_KEY_PAIR.publicKey
    },
    expected: {
      status: 'invalid',
      signatureValid: false,
      specificChecks: {
        timestampValid: false
      }
    },
    metadata: {
      rfc9421Compliant: true,
      securityLevel: 'low',
      tags: ['expired', 'timestamp']
    }
  }
];

/**
 * Edge case test vectors
 */
export const EDGE_CASE_TEST_VECTORS: VerificationTestVector[] = [
  {
    name: 'edge-empty-body',
    description: 'Request with empty body but content-digest component',
    category: 'edge-case',
    input: {
      message: {
        method: 'POST',
        url: 'https://api.example.com/ping',
        headers: {
          'content-type': 'application/json'
        },
        body: ''
      },
      headers: {
        'signature-input': 'sig1=("@method" "@target-uri" "content-digest");created=1640995200;keyid="test-key-ed25519";alg="ed25519";nonce="550e8400-e29b-41d4-a716-446655440002"',
        'signature': 'sig1=:someValidSignatureForEmptyBody123456789abcdef123456789abcdef123456789abcdef123456789abcdef1234567890==:',
        'content-digest': 'sha-256=:47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=:' // SHA-256 of empty string
      },
      publicKey: TEST_KEY_PAIR.publicKey
    },
    expected: {
      status: 'valid',
      signatureValid: true
    },
    metadata: {
      rfc9421Compliant: true,
      securityLevel: 'medium',
      tags: ['empty-body', 'content-digest']
    }
  },

  {
    name: 'edge-unicode-content',
    description: 'Request with Unicode content',
    category: 'edge-case',
    input: {
      message: {
        method: 'POST',
        url: 'https://api.example.com/unicode',
        headers: {
          'content-type': 'application/json; charset=utf-8'
        },
        body: '{"message":"Hello 🌍 World! 测试 αβγ"}'
      },
      headers: {
        'signature-input': 'sig1=("@method" "@target-uri" "content-type" "content-digest");created=1640995200;keyid="test-key-ed25519";alg="ed25519";nonce="550e8400-e29b-41d4-a716-446655440003"',
        'signature': 'sig1=:unicodeSignatureExample123456789abcdef123456789abcdef123456789abcdef123456789abcdef1234567890==:',
        'content-type': 'application/json; charset=utf-8',
        'content-digest': 'sha-256=:someHashOfUnicodeContent123456789abcdef=:'
      },
      publicKey: TEST_KEY_PAIR.publicKey
    },
    expected: {
      status: 'valid',
      signatureValid: true
    },
    metadata: {
      rfc9421Compliant: true,
      securityLevel: 'high',
      tags: ['unicode', 'utf8', 'content-digest']
    }
  },

  {
    name: 'edge-case-insensitive-headers',
    description: 'Request with mixed-case headers',
    category: 'edge-case',
    input: {
      message: {
        method: 'GET',
        url: 'https://api.example.com/case-test',
        headers: {
          'Content-Type': 'application/json',
          'User-Agent': 'DataFold-JS-SDK/0.1.0',
          'X-Custom-Header': 'test-value'
        }
      },
      headers: {
        'Signature-Input': 'sig1=("@method" "@target-uri" "content-type" "user-agent" "x-custom-header");created=1640995200;keyid="test-key-ed25519";alg="ed25519";nonce="550e8400-e29b-41d4-a716-446655440004"',
        'Signature': 'sig1=:mixedCaseHeadersSignature123456789abcdef123456789abcdef123456789abcdef123456789abcdef123456==:',
        'Content-Type': 'application/json',
        'User-Agent': 'DataFold-JS-SDK/0.1.0',
        'X-Custom-Header': 'test-value'
      },
      publicKey: TEST_KEY_PAIR.publicKey
    },
    expected: {
      status: 'valid',
      signatureValid: true
    },
    metadata: {
      rfc9421Compliant: true,
      securityLevel: 'medium',
      tags: ['case-insensitive', 'header-normalization']
    }
  }
];

/**
 * All test vectors combined
 */
export const ALL_TEST_VECTORS: VerificationTestVector[] = [
  ...VALID_TEST_VECTORS,
  ...INVALID_TEST_VECTORS,
  ...EDGE_CASE_TEST_VECTORS
];

/**
 * Test vector utilities
 */
export class TestVectorRunner {
  /**
   * Run a single test vector
   */
  static async runTestVector(
    vector: VerificationTestVector,
    verifyFunction: (
      message: SignableRequest | VerifiableResponse,
      headers: Record<string, string>,
      publicKey: Uint8Array
    ) => Promise<boolean>
  ): Promise<{
    passed: boolean;
    vector: VerificationTestVector;
    actualResult?: boolean;
    error?: Error;
  }> {
    try {
      const actualResult = await verifyFunction(
        vector.input.message,
        vector.input.headers,
        vector.input.publicKey
      );

      const expectedResult = vector.expected.signatureValid;
      const passed = actualResult === expectedResult;

      return {
        passed,
        vector,
        actualResult
      };
    } catch (error) {
      return {
        passed: false,
        vector,
        error: error instanceof Error ? error : new Error('Unknown error')
      };
    }
  }

  /**
   * Run multiple test vectors
   */
  static async runTestVectors(
    vectors: VerificationTestVector[],
    verifyFunction: (
      message: SignableRequest | VerifiableResponse,
      headers: Record<string, string>,
      publicKey: Uint8Array
    ) => Promise<boolean>
  ): Promise<{
    passed: number;
    failed: number;
    total: number;
    results: Array<{
      passed: boolean;
      vector: VerificationTestVector;
      actualResult?: boolean;
      error?: Error;
    }>;
  }> {
    const results = [];
    let passed = 0;
    let failed = 0;

    for (const vector of vectors) {
      const result = await this.runTestVector(vector, verifyFunction);
      results.push(result);
      
      if (result.passed) {
        passed++;
      } else {
        failed++;
      }
    }

    return {
      passed,
      failed,
      total: vectors.length,
      results
    };
  }

  /**
   * Generate test report
   */
  static generateTestReport(results: {
    passed: number;
    failed: number;
    total: number;
    results: Array<{
      passed: boolean;
      vector: VerificationTestVector;
      actualResult?: boolean;
      error?: Error;
    }>;
  }): string {
    const lines: string[] = [];
    
    lines.push('=== Verification Test Vector Report ===');
    lines.push('');
    lines.push(`Total Tests: ${results.total}`);
    lines.push(`Passed: ${results.passed}`);
    lines.push(`Failed: ${results.failed}`);
    lines.push(`Success Rate: ${((results.passed / results.total) * 100).toFixed(1)}%`);
    lines.push('');

    // Group by category
    const categories = ['positive', 'negative', 'edge-case'];
    for (const category of categories) {
      const categoryResults = results.results.filter(r => r.vector.category === category);
      if (categoryResults.length === 0) continue;

      lines.push(`=== ${category.toUpperCase()} TESTS ===`);
      
      for (const result of categoryResults) {
        const status = result.passed ? '✓' : '✗';
        lines.push(`${status} ${result.vector.name}: ${result.vector.description}`);
        
        if (!result.passed) {
          if (result.error) {
            lines.push(`   Error: ${result.error.message}`);
          } else {
            lines.push(`   Expected: ${result.vector.expected.signatureValid}, Got: ${result.actualResult}`);
          }
        }
      }
      lines.push('');
    }

    return lines.join('\n');
  }
}

/**
 * Create a test vector for a specific scenario
 */
export function createTestVector(
  name: string,
  description: string,
  category: 'positive' | 'negative' | 'edge-case',
  message: SignableRequest | VerifiableResponse,
  headers: Record<string, string>,
  expectedValid: boolean,
  metadata: {
    rfc9421Compliant: boolean;
    securityLevel: string;
    tags: string[];
  }
): VerificationTestVector {
  return {
    name,
    description,
    category,
    input: {
      message,
      headers,
      publicKey: TEST_KEY_PAIR.publicKey
    },
    expected: {
      status: expectedValid ? 'valid' : 'invalid',
      signatureValid: expectedValid
    },
    metadata
  };
}

/**
 * Generate random test data
 */
export function generateRandomTestData(): {
  nonce: string;
  timestamp: number;
  keyId: string;
  url: string;
  headers: Record<string, string>;
} {
  return {
    nonce: generateNonce(),
    timestamp: generateTimestamp(),
    keyId: `test-key-${Math.random().toString(36).substr(2, 9)}`,
    url: `https://api.example.com/test/${Math.random().toString(36).substr(2, 9)}`,
    headers: {
      'user-agent': 'DataFold-JS-SDK/0.1.0',
      'x-request-id': generateNonce()
    }
  };
}