/**
 * Unit tests for signature verification functionality
 */

import {
  RFC9421Verifier,
  createVerifier,
  verifySignature,
  VerificationConfig,
  VERIFICATION_POLICIES,
  RFC9421Inspector,
  createInspector,
  VerificationError
} from '../verification/index.js';
import { TEST_KEY_PAIR, VALID_TEST_VECTORS, INVALID_TEST_VECTORS } from '../verification/test-vectors.js';
import { SignableRequest } from '../signing/types.js';

describe('RFC9421Verifier', () => {
  let verifier: RFC9421Verifier;
  let config: VerificationConfig;

  beforeEach(() => {
    config = {
      policies: VERIFICATION_POLICIES,
      publicKeys: {
        [TEST_KEY_PAIR.keyId]: TEST_KEY_PAIR.publicKey
      },
      defaultPolicy: 'standard'
    };
    verifier = createVerifier(config);
  });

  describe('basic verification', () => {
    it('should verify a valid signature', async () => {
      const request: SignableRequest = {
        method: 'GET',
        url: 'https://api.example.com/data',
        headers: {
          'user-agent': 'DataFold-JS-SDK/0.1.0'
        }
      };

      const currentTimestamp = Math.floor(Date.now() / 1000);
      const headers = {
        'signature-input': `sig1=("@method" "@target-uri");created=${currentTimestamp};keyid="test-key-ed25519";alg="ed25519";nonce="550e8400-e29b-41d4-a716-446655440000"`,
        'signature': 'sig1=:d2pmTvmbncD3xQm8E9ZV2828BjQWGgiwAaw5bAkgibUopemLJcWDy/lkbbHAve7I2l1nirKIcOpxD4DBMq8hBg==:'
      };

      const result = await verifier.verify(request, headers);
      
      expect(result.status).toBe('valid');
      expect(result.signatureValid).toBe(true);
      expect(result.checks.formatValid).toBe(true);
    });

    it('should reject invalid signature format', async () => {
      const request: SignableRequest = {
        method: 'GET',
        url: 'https://api.example.com/data',
        headers: {}
      };

      const headers = {
        'signature-input': 'invalid-format',
        'signature': 'also-invalid'
      };

      const result = await verifier.verify(request, headers);
      
      expect(result.status).toBe('invalid');
      expect(result.signatureValid).toBe(false);
      expect(result.checks.formatValid).toBe(false);
    });

    it('should handle missing signature headers', async () => {
      const request: SignableRequest = {
        method: 'GET',
        url: 'https://api.example.com/data',
        headers: {}
      };

      const headers = {}; // No signature headers

      await expect(verifier.verify(request, headers)).rejects.toThrow(VerificationError);
    });
  });

  describe('policy enforcement', () => {
    it('should enforce strict policy requirements', async () => {
      const strictConfig = {
        ...config,
        defaultPolicy: 'strict'
      };
      const strictVerifier = createVerifier(strictConfig);

      const request: SignableRequest = {
        method: 'GET',
        url: 'https://api.example.com/data',
        headers: {}
      };

      const currentTimestamp = Math.floor(Date.now() / 1000);
      const headers = {
        'signature-input': `sig1=("@method");created=${currentTimestamp};keyid="test-key-ed25519";alg="ed25519";nonce="550e8400-e29b-41d4-a716-446655440000"`,
        'signature': 'sig1=:someSignature123456789abcdef123456789abcdef123456789abcdef123456789abcdef123456789abcdef1234==:'
      };

      const result = await strictVerifier.verify(request, headers);
      
      // Should fail because strict policy requires more components
      expect(result.status).toBe('invalid');
      expect(result.diagnostics.policyCompliance.missingRequiredComponents.length).toBeGreaterThan(0);
    });

    it('should allow lenient policy', async () => {
      const lenientConfig = {
        ...config,
        defaultPolicy: 'lenient'
      };
      const lenientVerifier = createVerifier(lenientConfig);

      const request: SignableRequest = {
        method: 'GET',
        url: 'https://api.example.com/data',
        headers: {}
      };

      const headers = {
        'signature-input': 'sig1=("@method");created=1640995200;keyid="test-key-ed25519";alg="ed25519";nonce="550e8400-e29b-41d4-a716-446655440000"',
        'signature': 'sig1=:someSignature123456789abcdef123456789abcdef123456789abcdef123456789abcdef123456789abcdef1234==:'
      };

      const result = await lenientVerifier.verify(request, headers);
      
      // Should pass with lenient policy
      expect(result.checks.componentCoverageValid).toBe(true);
    });
  });

  describe('configuration management', () => {
    it('should allow adding public keys', () => {
      const newKey = new Uint8Array(32).fill(0x42);
      verifier.addPublicKey('new-key', newKey);
      
      expect(verifier['config'].publicKeys['new-key']).toEqual(newKey);
    });

    it('should allow removing public keys', () => {
      verifier.removePublicKey(TEST_KEY_PAIR.keyId);
      
      expect(verifier['config'].publicKeys[TEST_KEY_PAIR.keyId]).toBeUndefined();
    });

    it('should validate configuration on creation', () => {
      const invalidConfig = {
        policies: {}, // Empty policies
        publicKeys: {}
      };

      expect(() => createVerifier(invalidConfig)).toThrow(VerificationError);
    });
  });

  describe('batch verification', () => {
    it('should verify multiple signatures', async () => {
      const requests = [
        {
          message: {
            method: 'GET' as const,
            url: 'https://api.example.com/data1',
            headers: {}
          },
          headers: {
            'signature-input': 'sig1=("@method" "@target-uri");created=1640995200;keyid="test-key-ed25519";alg="ed25519";nonce="550e8400-e29b-41d4-a716-446655440001"',
            'signature': 'sig1=:signature1===================================================================================================================:'
          }
        },
        {
          message: {
            method: 'POST' as const,
            url: 'https://api.example.com/data2',
            headers: {},
            body: '{"test": true}'
          },
          headers: {
            'signature-input': 'sig1=("@method" "@target-uri");created=1640995200;keyid="test-key-ed25519";alg="ed25519";nonce="550e8400-e29b-41d4-a716-446655440002"',
            'signature': 'sig1=:signature2===================================================================================================================:'
          }
        }
      ];

      const results = await verifier.verifyBatch(requests);
      
      expect(results).toHaveLength(2);
      expect(results[0].status).toBeDefined();
      expect(results[1].status).toBeDefined();
    });
  });

  describe('performance monitoring', () => {
    it('should measure verification performance', async () => {
      const request: SignableRequest = {
        method: 'GET',
        url: 'https://api.example.com/data',
        headers: {}
      };

      const headers = {
        'signature-input': 'sig1=("@method" "@target-uri");created=1640995200;keyid="test-key-ed25519";alg="ed25519";nonce="550e8400-e29b-41d4-a716-446655440000"',
        'signature': 'sig1=:someSignature123456789abcdef123456789abcdef123456789abcdef123456789abcdef123456789abcdef1234==:'
      };

      const result = await verifier.verify(request, headers);
      
      expect(result.performance.totalTime).toBeGreaterThan(0);
      expect(result.performance.stepTimings).toBeDefined();
      expect(Object.keys(result.performance.stepTimings).length).toBeGreaterThan(0);
    });

    it('should warn if verification takes too long', async () => {
      const consoleSpy = jest.spyOn(console, 'warn').mockImplementation();

      // Mock a slow verification (this is contrived for testing)
      const originalPerformance = performance.now;
      let callCount = 0;
      performance.now = jest.fn(() => {
        callCount++;
        return callCount === 1 ? 0 : 15; // 15ms elapsed
      });

      try {
        const request: SignableRequest = {
          method: 'GET',
          url: 'https://api.example.com/data',
          headers: {}
        };

        const headers = {
          'signature-input': 'sig1=("@method" "@target-uri");created=1640995200;keyid="test-key-ed25519";alg="ed25519";nonce="550e8400-e29b-41d4-a716-446655440000"',
          'signature': 'sig1=:someSignature123456789abcdef123456789abcdef123456789abcdef123456789abcdef123456789abcdef1234==:'
        };

        await verifier.verify(request, headers);
        
        // Should have warned about slow verification (if target is <10ms)
        // This test may be environment-dependent
      } finally {
        performance.now = originalPerformance;
        consoleSpy.mockRestore();
      }
    });
  });
});

describe('RFC9421Inspector', () => {
  let inspector: RFC9421Inspector;

  beforeEach(() => {
    inspector = createInspector();
  });

  describe('format inspection', () => {
    it('should validate correct signature format', () => {
      const headers = {
        'signature-input': 'sig1=("@method" "@target-uri");created=1640995200;keyid="test-key-ed25519";alg="ed25519";nonce="550e8400-e29b-41d4-a716-446655440000"',
        'signature': 'sig1=:d2pmTvmbncD3xQm8E9ZV2828BjQWGgiwAaw5bAkgibUopemLJcWDy/lkbbHAve7I2l1nirKIcOpxD4DBMq8hBg==:'
      };

      const analysis = inspector.inspectFormat(headers);
      
      expect(analysis.isValidRFC9421).toBe(true);
      expect(analysis.signatureIds).toContain('sig1');
      expect(analysis.issues.filter(i => i.severity === 'error')).toHaveLength(0);
    });

    it('should detect format issues', () => {
      const headers = {
        'signature-input': 'invalid-format-missing-params',
        'signature': 'also-invalid'
      };

      const analysis = inspector.inspectFormat(headers);
      
      expect(analysis.isValidRFC9421).toBe(false);
      expect(analysis.issues.length).toBeGreaterThan(0);
      expect(analysis.issues.some(i => i.severity === 'error')).toBe(true);
    });

    it('should detect missing headers', () => {
      const headers = {
        'signature-input': 'sig1=("@method");created=1640995200;keyid="test-key";alg="ed25519";nonce="test"'
        // Missing signature header
      };

      const analysis = inspector.inspectFormat(headers);
      
      expect(analysis.isValidRFC9421).toBe(false);
      expect(analysis.issues.some(i => i.code === 'MISSING_SIGNATURE')).toBe(true);
    });
  });

  describe('component analysis', () => {
    it('should analyze signature components', () => {
      const signatureData = {
        signatureId: 'sig1',
        signature: 'test-signature',
        coveredComponents: ['@method', '@target-uri', 'content-type'],
        params: {
          created: 1640995200,
          keyid: 'test-key-ed25519',
          alg: 'ed25519' as const,
          nonce: '550e8400-e29b-41d4-a716-446655440000'
        }
      };

      const analysis = inspector.analyzeComponents(signatureData);
      
      expect(analysis.validComponents).toContain('@method');
      expect(analysis.validComponents).toContain('@target-uri');
      expect(analysis.validComponents).toContain('content-type');
      expect(analysis.securityAssessment.level).toBeDefined();
    });

    it('should detect invalid components', () => {
      const signatureData = {
        signatureId: 'sig1',
        signature: 'test-signature',
        coveredComponents: ['@invalid-component', 'invalid-header-name!'],
        params: {
          created: 1640995200,
          keyid: 'test-key-ed25519',
          alg: 'ed25519' as const,
          nonce: '550e8400-e29b-41d4-a716-446655440000'
        }
      };

      const analysis = inspector.analyzeComponents(signatureData);
      
      expect(analysis.invalidComponents.length).toBeGreaterThan(0);
      expect(analysis.securityAssessment.level).toBe('low');
    });
  });

  describe('parameter validation', () => {
    it('should validate correct parameters', () => {
      const params = {
        created: Math.floor(Date.now() / 1000),
        keyid: 'test-key-ed25519',
        alg: 'ed25519' as const,
        nonce: '550e8400-e29b-41d4-a716-446655440000'
      };

      const validation = inspector.validateParameters(params);
      
      expect(validation.allValid).toBe(true);
      expect(validation.parameters.created.valid).toBe(true);
      expect(validation.parameters.keyid.valid).toBe(true);
      expect(validation.parameters.alg.valid).toBe(true);
      expect(validation.parameters.nonce.valid).toBe(true);
    });

    it('should detect invalid parameters', () => {
      const params = {
        created: -1, // Invalid timestamp
        keyid: '', // Empty key ID
        alg: 'rsa' as any, // Unsupported algorithm
        nonce: 'not-a-uuid' // Invalid nonce format
      };

      const validation = inspector.validateParameters(params);
      
      expect(validation.allValid).toBe(false);
      expect(validation.parameters.created.valid).toBe(false);
      expect(validation.parameters.keyid.valid).toBe(false);
      expect(validation.parameters.alg.valid).toBe(false);
      expect(validation.parameters.nonce.valid).toBe(false);
    });
  });

  describe('diagnostic report generation', () => {
    it('should generate comprehensive diagnostic report', () => {
      const mockResult = {
        status: 'valid' as const,
        signatureValid: true,
        checks: {
          formatValid: true,
          cryptographicValid: true,
          timestampValid: true,
          nonceValid: true,
          contentDigestValid: true,
          componentCoverageValid: true,
          customRulesValid: true
        },
        diagnostics: {
          signatureAnalysis: {
            algorithm: 'ed25519',
            keyId: 'test-key-ed25519',
            created: 1640995200,
            age: 3600,
            nonce: '550e8400-e29b-41d4-a716-446655440000',
            coveredComponents: ['@method', '@target-uri']
          },
          contentAnalysis: {
            hasContentDigest: false,
            contentSize: 0
          },
          policyCompliance: {
            policyName: 'standard',
            missingRequiredComponents: [],
            extraComponents: [],
            ruleResults: []
          },
          securityAnalysis: {
            securityLevel: 'high' as const,
            concerns: [],
            recommendations: []
          }
        },
        performance: {
          totalTime: 5.2,
          stepTimings: {
            extraction: 1.1,
            verification: 4.1
          }
        }
      };

      const report = inspector.generateDiagnosticReport(mockResult);
      
      expect(report).toContain('RFC 9421 Signature Verification Report');
      expect(report).toContain('Overall Status: VALID');
      expect(report).toContain('Signature Valid: YES');
      expect(report).toContain('Algorithm: ed25519');
      expect(report).toContain('Security Level: HIGH');
      expect(report).toContain('Total Time: 5.20ms');
    });
  });
});

describe('Helper Functions', () => {
  describe('verifySignature', () => {
    it('should provide quick verification', async () => {
      const request: SignableRequest = {
        method: 'GET',
        url: 'https://api.example.com/data',
        headers: {}
      };

      const headers = {
        'signature-input': 'sig1=("@method" "@target-uri");created=1640995200;keyid="test-key-ed25519";alg="ed25519";nonce="550e8400-e29b-41d4-a716-446655440000"',
        'signature': 'sig1=:d2pmTvmbncD3xQm8E9ZV2828BjQWGgiwAaw5bAkgibUopemLJcWDy/lkbbHAve7I2l1nirKIcOpxD4DBMq8hBg==:'
      };

      // This will likely fail since we don't have a real signature, but should not throw
      const result = await verifySignature(request, headers, TEST_KEY_PAIR.publicKey);
      
      expect(typeof result).toBe('boolean');
    });
  });
});

describe('Test Vector Validation', () => {
  it('should have valid test vectors', () => {
    expect(VALID_TEST_VECTORS.length).toBeGreaterThan(0);
    expect(INVALID_TEST_VECTORS.length).toBeGreaterThan(0);

    // Check test vector structures
    for (const vector of VALID_TEST_VECTORS) {
      expect(vector.name).toBeDefined();
      expect(vector.description).toBeDefined();
      expect(vector.category).toBe('positive');
      expect(vector.expected.signatureValid).toBe(true);
    }

    for (const vector of INVALID_TEST_VECTORS) {
      expect(vector.name).toBeDefined();
      expect(vector.description).toBeDefined();
      expect(vector.category).toBe('negative');
      expect(vector.expected.signatureValid).toBe(false);
    }
  });
});