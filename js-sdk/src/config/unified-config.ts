/**
 * Unified configuration management for JavaScript SDK
 * 
 * Provides cross-platform configuration loading and environment-specific
 * settings compatible with the DataFold unified configuration format.
 */

import { SigningConfig, SignatureComponents } from '../signing/types.js';
import { SecurityProfile } from '../signing/signing-config.js';

/**
 * Unified configuration structure
 */
export interface UnifiedConfig {
  config_format_version: string;
  environments: Record<string, EnvironmentConfig>;
  security_profiles: Record<string, UnifiedSecurityProfile>;
  defaults: DefaultConfig;
}

/**
 * Environment-specific configuration
 */
export interface EnvironmentConfig {
  signing: UnifiedSigningConfig;
  verification: VerificationConfig;
  logging: LoggingConfig;
  authentication: AuthenticationConfig;
  performance: PerformanceConfig;
}

/**
 * Unified signing configuration
 */
export interface UnifiedSigningConfig {
  policy: string;
  timeout_ms: number;
  required_components: string[];
  include_content_digest: boolean;
  include_timestamp: boolean;
  include_nonce: boolean;
  max_body_size_mb: number;
  debug: DebugConfig;
}

/**
 * Verification configuration
 */
export interface VerificationConfig {
  strict_timing: boolean;
  allow_clock_skew_seconds: number;
  require_nonce: boolean;
  max_signature_age_seconds: number;
}

/**
 * Logging configuration
 */
export interface LoggingConfig {
  level: string;
  colored_output: boolean;
  structured: boolean;
}

/**
 * Authentication configuration
 */
export interface AuthenticationConfig {
  store_tokens: boolean;
  auto_update_check: boolean;
  prompt_on_first_sign: boolean;
}

/**
 * Performance configuration
 */
export interface PerformanceConfig {
  cache_keys: boolean;
  max_concurrent_signs: number;
  default_timeout_secs: number;
  default_max_retries: number;
}

/**
 * Debug configuration
 */
export interface DebugConfig {
  enabled: boolean;
  log_canonical_strings: boolean;
  log_components: boolean;
  log_timing: boolean;
}

/**
 * Unified security profile
 */
export interface UnifiedSecurityProfile {
  description: string;
  required_components: string[];
  include_content_digest: boolean;
  digest_algorithm: string;
  validate_nonces: boolean;
  allow_custom_nonces: boolean;
}

/**
 * Default configuration values
 */
export interface DefaultConfig {
  environment: string;
  signing_mode: string;
  output_format: string;
  verbosity: number;
}

/**
 * Configuration loading error
 */
export class UnifiedConfigError extends Error {
  constructor(message: string, public readonly code?: string) {
    super(message);
    this.name = 'UnifiedConfigError';
  }
}

/**
 * Unified configuration manager for JavaScript SDK
 */
export class UnifiedConfigManager {
  private config: UnifiedConfig;
  private currentEnvironment: string;

  constructor(config: UnifiedConfig, environment?: string) {
    this.config = config;
    this.currentEnvironment = environment || config.defaults.environment;
    this.validate();
  }

  /**
   * Load unified configuration from JSON string
   */
  static fromJSON(jsonString: string, environment?: string): UnifiedConfigManager {
    try {
      const config = JSON.parse(jsonString) as UnifiedConfig;
      return new UnifiedConfigManager(config, environment);
    } catch (error) {
      throw new UnifiedConfigError(
        `Failed to parse configuration JSON: ${error instanceof Error ? error.message : 'Unknown error'}`,
        'PARSE_ERROR'
      );
    }
  }

  /**
   * Load unified configuration from URL
   */
  static async fromURL(url: string, environment?: string): Promise<UnifiedConfigManager> {
    try {
      const response = await fetch(url);
      if (!response.ok) {
        throw new UnifiedConfigError(`Failed to fetch config: ${response.statusText}`, 'FETCH_ERROR');
      }
      const jsonString = await response.text();
      return UnifiedConfigManager.fromJSON(jsonString, environment);
    } catch (error) {
      if (error instanceof UnifiedConfigError) {
        throw error;
      }
      throw new UnifiedConfigError(
        `Failed to load configuration from URL: ${error instanceof Error ? error.message : 'Unknown error'}`,
        'LOAD_ERROR'
      );
    }
  }

  /**
   * Load default configuration (from relative path)
   */
  static async loadDefault(environment?: string): Promise<UnifiedConfigManager> {
    // In browser environments, this would need to be served by the application
    // In Node.js environments, this could read from the file system
    const defaultPath = '/config/unified-datafold-config.json';
    return UnifiedConfigManager.fromURL(defaultPath, environment);
  }

  /**
   * Set current environment
   */
  setEnvironment(environment: string): void {
    if (!this.config.environments[environment]) {
      throw new UnifiedConfigError(`Environment '${environment}' not found`, 'ENVIRONMENT_NOT_FOUND');
    }
    this.currentEnvironment = environment;
  }

  /**
   * Get current environment configuration
   */
  getCurrentEnvironmentConfig(): EnvironmentConfig {
    const envConfig = this.config.environments[this.currentEnvironment];
    if (!envConfig) {
      throw new UnifiedConfigError(`Environment '${this.currentEnvironment}' not found`, 'ENVIRONMENT_NOT_FOUND');
    }
    return envConfig;
  }

  /**
   * Convert to JavaScript SDK signing configuration
   */
  toSigningConfig(keyId: string, privateKey: Uint8Array): SigningConfig {
    const envConfig = this.getCurrentEnvironmentConfig();
    const profile = this.getSecurityProfile(envConfig.signing.policy);

    // Map required components to SignatureComponents format
    const components: SignatureComponents = {
      method: envConfig.signing.required_components.includes('@method'),
      targetUri: envConfig.signing.required_components.includes('@target-uri'),
      headers: envConfig.signing.required_components.filter(c => !c.startsWith('@')),
      contentDigest: envConfig.signing.include_content_digest
    };

    return {
      algorithm: 'ed25519' as const,
      keyId,
      privateKey,
      components,
      nonceGenerator: () => this.generateNonce(),
      timestampGenerator: () => this.generateTimestamp()
    };
  }

  /**
   * Get security profile by name
   */
  getSecurityProfile(name: string): UnifiedSecurityProfile {
    const profile = this.config.security_profiles[name];
    if (!profile) {
      throw new UnifiedConfigError(`Security profile '${name}' not found`, 'PROFILE_NOT_FOUND');
    }
    return profile;
  }

  /**
   * List available environments
   */
  listEnvironments(): string[] {
    return Object.keys(this.config.environments);
  }

  /**
   * List available security profiles
   */
  listSecurityProfiles(): string[] {
    return Object.keys(this.config.security_profiles);
  }

  /**
   * Get current environment name
   */
  getCurrentEnvironment(): string {
    return this.currentEnvironment;
  }

  /**
   * Get the full unified configuration
   */
  getConfig(): UnifiedConfig {
    return this.config;
  }

  /**
   * Get verification configuration for current environment
   */
  getVerificationConfig(): VerificationConfig {
    return this.getCurrentEnvironmentConfig().verification;
  }

  /**
   * Get logging configuration for current environment
   */
  getLoggingConfig(): LoggingConfig {
    return this.getCurrentEnvironmentConfig().logging;
  }

  /**
   * Get performance configuration for current environment
   */
  getPerformanceConfig(): PerformanceConfig {
    return this.getCurrentEnvironmentConfig().performance;
  }

  /**
   * Validate the configuration
   */
  private validate(): void {
    // Validate default environment exists
    if (!this.config.environments[this.config.defaults.environment]) {
      throw new UnifiedConfigError(
        `Default environment '${this.config.defaults.environment}' not found`,
        'INVALID_DEFAULT_ENVIRONMENT'
      );
    }

    // Validate each environment configuration
    for (const [envName, envConfig] of Object.entries(this.config.environments)) {
      // Validate signing policy references exist
      if (!this.config.security_profiles[envConfig.signing.policy]) {
        throw new UnifiedConfigError(
          `Environment '${envName}' references unknown security profile '${envConfig.signing.policy}'`,
          'INVALID_SECURITY_PROFILE'
        );
      }

      // Validate performance settings
      if (envConfig.performance.max_concurrent_signs <= 0) {
        throw new UnifiedConfigError(
          `Environment '${envName}' has invalid max_concurrent_signs`,
          'INVALID_PERFORMANCE_CONFIG'
        );
      }

      if (envConfig.signing.timeout_ms <= 0) {
        throw new UnifiedConfigError(
          `Environment '${envName}' has invalid signing timeout`,
          'INVALID_SIGNING_CONFIG'
        );
      }
    }
  }

  /**
   * Generate nonce for signing
   */
  private generateNonce(): string {
    const array = new Uint8Array(16);
    crypto.getRandomValues(array);
    return Array.from(array, byte => byte.toString(16).padStart(2, '0')).join('');
  }

  /**
   * Generate timestamp for signing
   */
  private generateTimestamp(): number {
    return Math.floor(Date.now() / 1000);
  }
}

/**
 * Create unified configuration manager from configuration object
 */
export function createUnifiedConfig(config: UnifiedConfig, environment?: string): UnifiedConfigManager {
  return new UnifiedConfigManager(config, environment);
}

/**
 * Load unified configuration from JSON string
 */
export function loadUnifiedConfigFromJSON(jsonString: string, environment?: string): UnifiedConfigManager {
  return UnifiedConfigManager.fromJSON(jsonString, environment);
}

/**
 * Load unified configuration from URL
 */
export async function loadUnifiedConfigFromURL(url: string, environment?: string): Promise<UnifiedConfigManager> {
  return UnifiedConfigManager.fromURL(url, environment);
}

/**
 * Load default unified configuration
 */
export async function loadDefaultUnifiedConfig(environment?: string): Promise<UnifiedConfigManager> {
  return UnifiedConfigManager.loadDefault(environment);
}