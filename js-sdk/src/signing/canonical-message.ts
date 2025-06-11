/**
 * Canonical message construction for RFC 9421 HTTP Message Signatures
 */

import { 
  SignableRequest, 
  SignatureComponents, 
  SignatureParams, 
  ContentDigest, 
  SigningError, 
  SigningContext 
} from './types.js';
import { 
  parseUrl, 
  normalizeHeaderName, 
  calculateContentDigest 
} from './utils.js';

/**
 * Canonical message builder for RFC 9421 signatures
 */
export class CanonicalMessageBuilder {
  private request: SignableRequest;
  private components: SignatureComponents;
  private params: SignatureParams;
  private contentDigest?: ContentDigest;

  constructor(context: SigningContext) {
    this.request = context.request;
    this.components = context.config.components;
    this.params = context.params;
    this.contentDigest = context.contentDigest;
  }

  /**
   * Build the canonical message for signing
   */
  async build(): Promise<string> {
    const lines: string[] = [];
    const coveredComponents: string[] = [];

    // Add @method component if enabled
    if (this.components.method) {
      const methodLine = this.buildMethodComponent();
      lines.push(methodLine);
      coveredComponents.push('@method');
    }

    // Add @target-uri component if enabled
    if (this.components.targetUri) {
      const targetUriLine = this.buildTargetUriComponent();
      lines.push(targetUriLine);
      coveredComponents.push('@target-uri');
    }

    // Add header components
    if (this.components.headers && this.components.headers.length > 0) {
      for (const headerName of this.components.headers) {
        const headerLine = this.buildHeaderComponent(headerName);
        lines.push(headerLine);
        coveredComponents.push(headerName.toLowerCase());
      }
    }

    // Add content-digest component if enabled and request has body
    if (this.components.contentDigest && this.hasRequestBody()) {
      if (!this.contentDigest) {
        throw new SigningError(
          'Content digest required but not provided',
          'MISSING_CONTENT_DIGEST'
        );
      }
      const digestLine = this.buildContentDigestComponent();
      lines.push(digestLine);
      coveredComponents.push('content-digest');
    }

    // Add @signature-params component (always last)
    const signatureParamsLine = this.buildSignatureParamsComponent(coveredComponents);
    lines.push(signatureParamsLine);

    return lines.join('\n');
  }

  /**
   * Build @method component
   */
  private buildMethodComponent(): string {
    return `"@method": ${this.request.method}`;
  }

  /**
   * Build @target-uri component
   */
  private buildTargetUriComponent(): string {
    const { targetUri } = parseUrl(this.request.url);
    return `"@target-uri": ${targetUri}`;
  }

  /**
   * Build header component
   */
  private buildHeaderComponent(headerName: string): string {
    const normalizedName = normalizeHeaderName(headerName);
    const headerValue = this.getHeaderValue(normalizedName);
    
    return `"${normalizedName}": ${headerValue}`;
  }

  /**
   * Build content-digest component
   */
  private buildContentDigestComponent(): string {
    if (!this.contentDigest) {
      throw new SigningError(
        'Content digest not available',
        'MISSING_CONTENT_DIGEST'
      );
    }
    
    return `"content-digest": ${this.contentDigest.headerValue}`;
  }

  /**
   * Build @signature-params component
   */
  private buildSignatureParamsComponent(coveredComponents: string[]): string {
    // Format covered components as quoted strings
    const componentsList = coveredComponents
      .map(component => `"${component}"`)
      .join(' ');

    // Build parameters string
    const params = [
      `created=${this.params.created}`,
      `keyid="${this.params.keyid}"`,
      `alg="${this.params.alg}"`,
      `nonce="${this.params.nonce}"`
    ].join(';');

    return `"@signature-params": (${componentsList});${params}`;
  }

  /**
   * Get header value from request
   */
  private getHeaderValue(headerName: string): string {
    const value = this.request.headers[headerName] || 
                  this.request.headers[headerName.toLowerCase()] ||
                  this.request.headers[headerName.toUpperCase()];
    
    return value || '';
  }

  /**
   * Check if request has a body
   */
  private hasRequestBody(): boolean {
    return this.request.body !== undefined && this.request.body !== null;
  }
}

/**
 * Build canonical message from signing context
 */
export async function buildCanonicalMessage(context: SigningContext): Promise<string> {
  const builder = new CanonicalMessageBuilder(context);
  return builder.build();
}

/**
 * Validate canonical message format
 */
export function validateCanonicalMessage(message: string): boolean {
  if (!message || typeof message !== 'string') {
    return false;
  }

  const lines = message.split('\n');
  
  // Must have at least one component line plus @signature-params
  if (lines.length < 2) {
    return false;
  }

  // Last line must be @signature-params
  const lastLine = lines[lines.length - 1];
  if (!lastLine.startsWith('"@signature-params":')) {
    return false;
  }

  // All lines must follow component format: "name": value
  for (const line of lines) {
    if (!line.match(/^"[^"]+": .+$/)) {
      return false;
    }
  }

  return true;
}

/**
 * Extract covered components from canonical message
 */
export function extractCoveredComponents(message: string): string[] {
  const lines = message.split('\n');
  const components: string[] = [];

  for (const line of lines) {
    const match = line.match(/^"([^"]+)":/);
    if (match && match[1] !== '@signature-params') {
      components.push(match[1]);
    }
  }

  return components;
}

/**
 * Extract signature parameters from canonical message
 */
export function extractSignatureParams(message: string): Partial<SignatureParams> {
  const lines = message.split('\n');
  const lastLine = lines[lines.length - 1];
  
  if (!lastLine.startsWith('"@signature-params":')) {
    throw new SigningError(
      'Invalid canonical message format - missing @signature-params',
      'INVALID_CANONICAL_MESSAGE'
    );
  }

  // Extract parameters from the signature-params line
  // Format: "@signature-params": (components);created=123;keyid="key";alg="ed25519";nonce="uuid"
  const paramsMatch = lastLine.match(/\);(.+)$/);
  if (!paramsMatch) {
    throw new SigningError(
      'Invalid signature-params format',
      'INVALID_SIGNATURE_PARAMS'
    );
  }

  const paramsString = paramsMatch[1];
  const params: Partial<SignatureParams> = {};

  // Parse semicolon-separated parameters
  const paramPairs = paramsString.split(';');
  for (const pair of paramPairs) {
    const [key, value] = pair.split('=', 2);
    if (key && value) {
      const trimmedKey = key.trim();
      const trimmedValue = value.trim().replace(/^"|"$/g, ''); // Remove quotes

      switch (trimmedKey) {
        case 'created':
          params.created = parseInt(trimmedValue, 10);
          break;
        case 'keyid':
          params.keyid = trimmedValue;
          break;
        case 'alg':
          params.alg = trimmedValue as any;
          break;
        case 'nonce':
          params.nonce = trimmedValue;
          break;
      }
    }
  }

  return params;
}

/**
 * Build signature input string for Signature-Input header
 */
export function buildSignatureInput(
  signatureName: string,
  coveredComponents: string[],
  params: SignatureParams
): string {
  // Format covered components
  const componentsList = coveredComponents
    .map(component => `"${component}"`)
    .join(' ');

  // Format parameters
  const paramsList = [
    `created=${params.created}`,
    `keyid="${params.keyid}"`,
    `alg="${params.alg}"`,
    `nonce="${params.nonce}"`
  ].join(';');

  return `${signatureName}=(${componentsList});${paramsList}`;
}

/**
 * Parse signature input string from Signature-Input header
 */
export function parseSignatureInput(signatureInput: string): {
  signatureName: string;
  coveredComponents: string[];
  params: SignatureParams;
} {
  // Format: sig1=("@method" "@target-uri");created=123;keyid="key";alg="ed25519";nonce="uuid"
  
  const equalIndex = signatureInput.indexOf('=');
  if (equalIndex === -1) {
    throw new SigningError(
      'Invalid signature input format - missing equals sign',
      'INVALID_SIGNATURE_INPUT'
    );
  }

  const signatureName = signatureInput.substring(0, equalIndex);
  const definition = signatureInput.substring(equalIndex + 1);

  // Split into components and parameters
  const sections = definition.split(';');
  if (sections.length < 2) {
    throw new SigningError(
      'Invalid signature input format - missing parameters',
      'INVALID_SIGNATURE_INPUT'
    );
  }

  // Parse covered components
  const componentsSection = sections[0];
  const componentsMatch = componentsSection.match(/^\((.+)\)$/);
  if (!componentsMatch) {
    throw new SigningError(
      'Invalid components format in signature input',
      'INVALID_COMPONENTS_FORMAT'
    );
  }

  const coveredComponents = componentsMatch[1]
    .split(/\s+/)
    .map(component => component.replace(/^"|"$/g, '')); // Remove quotes

  // Parse parameters
  const params: any = {};
  for (let i = 1; i < sections.length; i++) {
    const [key, value] = sections[i].split('=', 2);
    if (key && value) {
      const trimmedKey = key.trim();
      const trimmedValue = value.trim().replace(/^"|"$/g, ''); // Remove quotes

      switch (trimmedKey) {
        case 'created':
          params.created = parseInt(trimmedValue, 10);
          break;
        case 'keyid':
          params.keyid = trimmedValue;
          break;
        case 'alg':
          params.alg = trimmedValue;
          break;
        case 'nonce':
          params.nonce = trimmedValue;
          break;
      }
    }
  }

  // Validate required parameters
  if (!params.created || !params.keyid || !params.alg || !params.nonce) {
    throw new SigningError(
      'Missing required signature parameters',
      'MISSING_SIGNATURE_PARAMS',
      { found: Object.keys(params) }
    );
  }

  return {
    signatureName,
    coveredComponents,
    params: params as SignatureParams
  };
}