/**
 * Signature inspection and debugging utilities
 */

import {
  SignatureInspector,
  SignatureFormatAnalysis,
  ComponentAnalysis,
  ParameterValidation,
  FormatIssue,
  ComponentIssue,
  ComponentSecurityAssessment,
  ExtractedSignatureData,
  VerificationResult,
  VerificationError,
  VerifiableResponse
} from './types.js';
import {
  SignatureParams,
  SignableRequest
} from '../signing/types.js';
import {
  parseSignatureInput,
  validateCanonicalMessage
} from '../signing/canonical-message.js';
import {
  validateTimestamp,
  validateNonce,
  validateHeaderName
} from '../signing/utils.js';

/**
 * RFC 9421 signature inspector for debugging and analysis
 */
export class RFC9421Inspector implements SignatureInspector {
  /**
   * Inspect signature format and structure
   */
  inspectFormat(headers: Record<string, string>): SignatureFormatAnalysis {
    const issues: FormatIssue[] = [];
    const signatureHeaders: string[] = [];
    const signatureIds: string[] = [];

    // Find signature-related headers (case-insensitive)
    const foundHeaders = this.findSignatureHeaders(headers);
    signatureHeaders.push(...Object.keys(foundHeaders));

    // Check for required headers
    if (!foundHeaders['signature-input']) {
      issues.push({
        severity: 'error',
        code: 'MISSING_SIGNATURE_INPUT',
        message: 'Signature-Input header is required',
        component: 'signature-input'
      });
    }

    if (!foundHeaders['signature']) {
      issues.push({
        severity: 'error',
        code: 'MISSING_SIGNATURE',
        message: 'Signature header is required',
        component: 'signature'
      });
    }

    // Analyze signature-input header
    if (foundHeaders['signature-input']) {
      try {
        const parsed = parseSignatureInput(foundHeaders['signature-input']);
        signatureIds.push(parsed.signatureName);
        
        // Validate signature input format
        this.validateSignatureInputFormat(foundHeaders['signature-input'], issues);
        
        // Validate parameters
        this.validateSignatureParameters(parsed.params, issues);
        
        // Validate covered components
        this.validateCoveredComponents(parsed.coveredComponents, issues);
        
      } catch (error) {
        issues.push({
          severity: 'error',
          code: 'INVALID_SIGNATURE_INPUT_FORMAT',
          message: `Failed to parse signature-input: ${error instanceof Error ? error.message : 'Unknown error'}`,
          component: 'signature-input'
        });
      }
    }

    // Analyze signature header
    if (foundHeaders['signature']) {
      this.validateSignatureHeaderFormat(foundHeaders['signature'], issues);
    }

    // Check for content-digest if present
    if (foundHeaders['content-digest']) {
      this.validateContentDigestFormat(foundHeaders['content-digest'], issues);
    }

    const isValidRFC9421 = issues.filter(issue => issue.severity === 'error').length === 0;

    return {
      isValidRFC9421,
      issues,
      signatureHeaders,
      signatureIds
    };
  }

  /**
   * Analyze signature components
   */
  analyzeComponents(signatureData: ExtractedSignatureData): ComponentAnalysis {
    const validComponents: string[] = [];
    const invalidComponents: ComponentIssue[] = [];
    const missingComponents: string[] = [];

    // Analyze each covered component
    for (const component of signatureData.coveredComponents) {
      const analysis = this.analyzeComponent(component);
      
      if (analysis.valid) {
        validComponents.push(component);
      } else {
        invalidComponents.push({
          component,
          type: 'format',
          message: analysis.message || 'Invalid component format'
        });
      }
    }

    // Check for recommended components
    const recommendedComponents = ['@method', '@target-uri'];
    for (const recommended of recommendedComponents) {
      if (!signatureData.coveredComponents.includes(recommended)) {
        missingComponents.push(recommended);
      }
    }

    // Generate security assessment
    const securityAssessment = this.assessComponentSecurity(signatureData.coveredComponents);

    return {
      validComponents,
      invalidComponents,
      missingComponents,
      securityAssessment
    };
  }

  /**
   * Validate signature parameters
   */
  validateParameters(params: SignatureParams): ParameterValidation {
    const validation: ParameterValidation = {
      allValid: true,
      parameters: {
        created: { valid: true },
        keyid: { valid: true },
        alg: { valid: true },
        nonce: { valid: true }
      },
      insights: []
    };

    // Validate created timestamp
    if (!validateTimestamp(params.created)) {
      validation.parameters.created = {
        valid: false,
        message: 'Invalid timestamp format or value'
      };
      validation.allValid = false;
    } else {
      const now = Math.floor(Date.now() / 1000);
      const age = now - params.created;
      
      if (age > 3600) {
        validation.insights.push(`Timestamp is ${Math.floor(age / 60)} minutes old`);
      }
      
      if (age < 0) {
        validation.insights.push('Timestamp is from the future (possible clock skew)');
      }
    }

    // Validate key ID
    if (!params.keyid || typeof params.keyid !== 'string' || params.keyid.trim() === '') {
      validation.parameters.keyid = {
        valid: false,
        message: 'Key ID must be a non-empty string'
      };
      validation.allValid = false;
    }

    // Validate algorithm
    if (params.alg !== 'ed25519') {
      validation.parameters.alg = {
        valid: false,
        message: `Unsupported algorithm: ${params.alg}`
      };
      validation.allValid = false;
    }

    // Validate nonce
    if (!validateNonce(params.nonce)) {
      validation.parameters.nonce = {
        valid: false,
        message: 'Invalid nonce format (should be UUID v4)'
      };
      validation.allValid = false;
    }

    return validation;
  }

  /**
   * Generate diagnostic report
   */
  generateDiagnosticReport(result: VerificationResult): string {
    const lines: string[] = [];
    
    lines.push('=== RFC 9421 Signature Verification Report ===');
    lines.push('');
    
    // Overall status
    lines.push(`Overall Status: ${result.status.toUpperCase()}`);
    lines.push(`Signature Valid: ${result.signatureValid ? 'YES' : 'NO'}`);
    lines.push('');
    
    // Individual checks
    lines.push('=== Individual Checks ===');
    for (const [check, passed] of Object.entries(result.checks)) {
      const status = passed ? '✓' : '✗';
      const label = check.replace(/([A-Z])/g, ' $1').toLowerCase();
      lines.push(`${status} ${label}`);
    }
    lines.push('');
    
    // Signature analysis
    const sig = result.diagnostics.signatureAnalysis;
    lines.push('=== Signature Analysis ===');
    lines.push(`Algorithm: ${sig.algorithm}`);
    lines.push(`Key ID: ${sig.keyId}`);
    lines.push(`Created: ${new Date(sig.created * 1000).toISOString()}`);
    lines.push(`Age: ${sig.age} seconds`);
    lines.push(`Nonce: ${sig.nonce}`);
    lines.push(`Covered Components: ${sig.coveredComponents.join(', ')}`);
    lines.push('');
    
    // Content analysis
    const content = result.diagnostics.contentAnalysis;
    lines.push('=== Content Analysis ===');
    lines.push(`Has Content Digest: ${content.hasContentDigest ? 'YES' : 'NO'}`);
    if (content.digestAlgorithm) {
      lines.push(`Digest Algorithm: ${content.digestAlgorithm}`);
    }
    lines.push(`Content Size: ${content.contentSize} bytes`);
    if (content.contentType) {
      lines.push(`Content Type: ${content.contentType}`);
    }
    lines.push('');
    
    // Policy compliance
    const policy = result.diagnostics.policyCompliance;
    lines.push('=== Policy Compliance ===');
    lines.push(`Policy: ${policy.policyName}`);
    if (policy.missingRequiredComponents.length > 0) {
      lines.push(`Missing Required Components: ${policy.missingRequiredComponents.join(', ')}`);
    }
    if (policy.extraComponents.length > 0) {
      lines.push(`Extra Components: ${policy.extraComponents.join(', ')}`);
    }
    
    // Custom rule results
    if (policy.ruleResults.length > 0) {
      lines.push('');
      lines.push('=== Custom Rule Results ===');
      for (const ruleResult of policy.ruleResults) {
        const status = ruleResult.passed ? '✓' : '✗';
        lines.push(`${status} ${ruleResult.message || 'Rule validation'}`);
      }
    }
    
    // Security analysis
    const security = result.diagnostics.securityAnalysis;
    lines.push('');
    lines.push('=== Security Analysis ===');
    lines.push(`Security Level: ${security.securityLevel.toUpperCase()}`);
    
    if (security.concerns.length > 0) {
      lines.push('');
      lines.push('Security Concerns:');
      for (const concern of security.concerns) {
        lines.push(`  - ${concern}`);
      }
    }
    
    if (security.recommendations.length > 0) {
      lines.push('');
      lines.push('Recommendations:');
      for (const recommendation of security.recommendations) {
        lines.push(`  - ${recommendation}`);
      }
    }
    
    // Performance
    lines.push('');
    lines.push('=== Performance ===');
    lines.push(`Total Time: ${result.performance.totalTime.toFixed(2)}ms`);
    if (Object.keys(result.performance.stepTimings).length > 0) {
      lines.push('Step Timings:');
      for (const [step, time] of Object.entries(result.performance.stepTimings)) {
        lines.push(`  - ${step}: ${time.toFixed(2)}ms`);
      }
    }
    
    // Error details
    if (result.error) {
      lines.push('');
      lines.push('=== Error Details ===');
      lines.push(`Code: ${result.error.code}`);
      lines.push(`Message: ${result.error.message}`);
      if (result.error.details) {
        lines.push('Details:');
        for (const [key, value] of Object.entries(result.error.details)) {
          lines.push(`  - ${key}: ${JSON.stringify(value)}`);
        }
      }
    }
    
    return lines.join('\n');
  }

  /**
   * Find signature-related headers (case-insensitive)
   */
  private findSignatureHeaders(headers: Record<string, string>): Record<string, string> {
    const found: Record<string, string> = {};
    const targetHeaders = ['signature-input', 'signature', 'content-digest'];
    
    for (const [key, value] of Object.entries(headers)) {
      const lowerKey = key.toLowerCase();
      if (targetHeaders.includes(lowerKey)) {
        found[lowerKey] = value;
      }
    }
    
    return found;
  }

  /**
   * Validate signature input format
   */
  private validateSignatureInputFormat(signatureInput: string, issues: FormatIssue[]): void {
    // Check basic format: sig1=("@method" "@target-uri");created=123;keyid="key"
    if (!signatureInput.includes('=') || !signatureInput.includes('(') || !signatureInput.includes(')')) {
      issues.push({
        severity: 'error',
        code: 'INVALID_FORMAT',
        message: 'Signature-Input header format is invalid',
        component: 'signature-input'
      });
      return;
    }

    // Check for required parameters
    const requiredParams = ['created', 'keyid', 'alg'];
    for (const param of requiredParams) {
      if (!signatureInput.includes(`${param}=`)) {
        issues.push({
          severity: 'error',
          code: 'MISSING_PARAMETER',
          message: `Missing required parameter: ${param}`,
          component: 'signature-input'
        });
      }
    }

    // Check component list format
    const componentMatch = signatureInput.match(/\(([^)]+)\)/);
    if (componentMatch) {
      const componentList = componentMatch[1];
      if (!componentList.includes('"')) {
        issues.push({
          severity: 'warning',
          code: 'UNQUOTED_COMPONENTS',
          message: 'Components should be quoted according to RFC 9421',
          component: 'signature-input'
        });
      }
    }
  }

  /**
   * Validate signature parameters
   */
  private validateSignatureParameters(params: SignatureParams, issues: FormatIssue[]): void {
    // Check timestamp
    if (!validateTimestamp(params.created)) {
      issues.push({
        severity: 'error',
        code: 'INVALID_TIMESTAMP',
        message: 'Invalid created timestamp',
        component: 'created'
      });
    }

    // Check nonce
    if (!validateNonce(params.nonce)) {
      issues.push({
        severity: 'warning',
        code: 'INVALID_NONCE_FORMAT',
        message: 'Nonce does not follow UUID v4 format',
        component: 'nonce'
      });
    }

    // Check algorithm
    if (params.alg !== 'ed25519') {
      issues.push({
        severity: 'warning',
        code: 'UNSUPPORTED_ALGORITHM',
        message: `Algorithm ${params.alg} is not ed25519`,
        component: 'alg'
      });
    }
  }

  /**
   * Validate covered components
   */
  private validateCoveredComponents(components: string[], issues: FormatIssue[]): void {
    if (components.length === 0) {
      issues.push({
        severity: 'error',
        code: 'NO_COMPONENTS',
        message: 'No signature components specified',
        component: 'components'
      });
      return;
    }

    for (const component of components) {
      if (component.startsWith('@')) {
        // Pseudo-component
        if (!['@method', '@target-uri', '@authority', '@scheme', '@request-target'].includes(component)) {
          issues.push({
            severity: 'warning',
            code: 'UNKNOWN_PSEUDO_COMPONENT',
            message: `Unknown pseudo-component: ${component}`,
            component: 'components'
          });
        }
      } else {
        // HTTP header
        if (!validateHeaderName(component)) {
          issues.push({
            severity: 'error',
            code: 'INVALID_HEADER_NAME',
            message: `Invalid header name: ${component}`,
            component: 'components'
          });
        }
      }
    }
  }

  /**
   * Validate signature header format
   */
  private validateSignatureHeaderFormat(signature: string, issues: FormatIssue[]): void {
    // Check format: sig1=:base64signature:
    const signaturePattern = /^[^=]+=:[A-Za-z0-9+/]+=*:$/;
    if (!signaturePattern.test(signature)) {
      issues.push({
        severity: 'error',
        code: 'INVALID_SIGNATURE_FORMAT',
        message: 'Signature header format is invalid (should be name=:base64:)',
        component: 'signature'
      });
    }

    // Check signature length for Ed25519 (88 base64 chars = 64 bytes)
    const base64Match = signature.match(/:([A-Za-z0-9+/]+=*):/);
    if (base64Match) {
      const base64Signature = base64Match[1];
      // Ed25519 signature is 64 bytes, which is 88 base64 chars (with padding)
      if (base64Signature.length !== 88) {
        issues.push({
          severity: 'warning',
          code: 'UNEXPECTED_SIGNATURE_LENGTH',
          message: `Signature length is ${base64Signature.length} base64 chars, expected 88 for Ed25519`,
          component: 'signature'
        });
      }
    }
  }

  /**
   * Validate content-digest format
   */
  private validateContentDigestFormat(contentDigest: string, issues: FormatIssue[]): void {
    // Check format: algorithm=:base64:
    const digestPattern = /^[^=]+=:[A-Za-z0-9+/]+=*:$/;
    if (!digestPattern.test(contentDigest)) {
      issues.push({
        severity: 'error',
        code: 'INVALID_DIGEST_FORMAT',
        message: 'Content-Digest header format is invalid (should be algorithm=:base64:)',
        component: 'content-digest'
      });
    }

    // Check algorithm
    const algMatch = contentDigest.match(/^([^=]+)=/);
    if (algMatch) {
      const algorithm = algMatch[1];
      if (!['sha-256', 'sha-512'].includes(algorithm)) {
        issues.push({
          severity: 'warning',
          code: 'UNSUPPORTED_DIGEST_ALGORITHM',
          message: `Digest algorithm ${algorithm} is not commonly supported`,
          component: 'content-digest'
        });
      }
    }
  }

  /**
   * Analyze individual component
   */
  private analyzeComponent(component: string): { valid: boolean; message?: string } {
    if (component.startsWith('@')) {
      // Pseudo-component
      const validPseudoComponents = ['@method', '@target-uri', '@authority', '@scheme', '@request-target'];
      if (validPseudoComponents.includes(component)) {
        return { valid: true };
      } else {
        return { valid: false, message: `Unknown pseudo-component: ${component}` };
      }
    } else {
      // HTTP header
      if (validateHeaderName(component)) {
        return { valid: true };
      } else {
        return { valid: false, message: `Invalid header name format: ${component}` };
      }
    }
  }

  /**
   * Assess security of component coverage
   */
  private assessComponentSecurity(components: string[]): ComponentSecurityAssessment {
    let score = 0;
    const strengths: string[] = [];
    const weaknesses: string[] = [];

    // Check for essential components
    if (components.includes('@method')) {
      score += 20;
      strengths.push('HTTP method is covered');
    } else {
      weaknesses.push('HTTP method is not covered');
    }

    if (components.includes('@target-uri')) {
      score += 20;
      strengths.push('Target URI is covered');
    } else {
      weaknesses.push('Target URI is not covered');
    }

    if (components.includes('content-digest')) {
      score += 25;
      strengths.push('Content integrity is protected');
    } else {
      weaknesses.push('Content integrity is not verified');
    }

    // Check for important headers
    if (components.includes('content-type')) {
      score += 10;
      strengths.push('Content type is covered');
    }

    if (components.includes('authorization')) {
      score += 15;
      strengths.push('Authorization header is covered');
    }

    if (components.includes('date') || components.includes('x-date')) {
      score += 10;
      strengths.push('Request timestamp is covered');
    }

    // Security level assessment
    let level: 'low' | 'medium' | 'high';
    if (score >= 80) {
      level = 'high';
    } else if (score >= 50) {
      level = 'medium';
    } else {
      level = 'low';
    }

    return {
      level,
      score,
      strengths,
      weaknesses
    };
  }
}

/**
 * Create a new signature inspector
 */
export function createInspector(): RFC9421Inspector {
  return new RFC9421Inspector();
}

/**
 * Quick signature format validation
 */
export function validateSignatureFormat(headers: Record<string, string>): boolean {
  const inspector = createInspector();
  const analysis = inspector.inspectFormat(headers);
  return analysis.isValidRFC9421;
}

/**
 * Generate a quick diagnostic summary
 */
export function quickDiagnostic(
  headers: Record<string, string>
): { valid: boolean; summary: string; issues: number } {
  const inspector = createInspector();
  const analysis = inspector.inspectFormat(headers);
  
  const errorCount = analysis.issues.filter(issue => issue.severity === 'error').length;
  const warningCount = analysis.issues.filter(issue => issue.severity === 'warning').length;
  
  let summary = '';
  if (analysis.isValidRFC9421) {
    summary = 'Valid RFC 9421 signature format';
    if (warningCount > 0) {
      summary += ` (${warningCount} warnings)`;
    }
  } else {
    summary = `Invalid signature format (${errorCount} errors`;
    if (warningCount > 0) {
      summary += `, ${warningCount} warnings`;
    }
    summary += ')';
  }

  return {
    valid: analysis.isValidRFC9421,
    summary,
    issues: analysis.issues.length
  };
}