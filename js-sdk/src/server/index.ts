/**
 * DataFold Server Integration Module
 * Export all server-related functionality
 */

// Export HTTP client
export * from './http-client.js';

// Export high-level integration
export * from './integration.js';

// Re-export relevant types
export type {
  ServerConnectionConfig,
  RetryConfig,
  DataFoldServerError,
  PublicKeyRegistrationRequest,
  PublicKeyRegistrationResponse,
  PublicKeyStatusResponse,
  SignatureVerificationRequest,
  SignatureVerificationResponse,
  ServerIntegrationInterface
} from '../types.js';

// Re-export enhanced HTTP client types
export type {
  SigningMode,
  EndpointSigningConfig,
  RequestInterceptor,
  ResponseInterceptor,
  RequestContext,
  SigningMetrics,
  HttpClientConfig
} from './http-client.js';