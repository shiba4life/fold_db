"""
Unified configuration management for Python SDK

Provides cross-platform configuration loading and environment-specific
settings compatible with the DataFold unified configuration format.
"""

import json
from typing import Dict, List, Optional, Any, Union
from dataclasses import dataclass
from pathlib import Path
import urllib.request
import urllib.error

from ..signing.types import SigningConfig, SignatureComponents, SignatureAlgorithm
from ..signing.utils import generate_nonce, generate_timestamp


class UnifiedConfigError(Exception):
    """Configuration loading and validation error"""
    
    def __init__(self, message: str, code: Optional[str] = None):
        super().__init__(message)
        self.code = code


@dataclass
class UnifiedSigningConfig:
    """Unified signing configuration"""
    policy: str
    timeout_ms: int
    required_components: List[str]
    include_content_digest: bool
    include_timestamp: bool
    include_nonce: bool
    max_body_size_mb: int
    debug: 'DebugConfig'


@dataclass
class VerificationConfig:
    """Verification configuration"""
    strict_timing: bool
    allow_clock_skew_seconds: int
    require_nonce: bool
    max_signature_age_seconds: int


@dataclass
class LoggingConfig:
    """Logging configuration"""
    level: str
    colored_output: bool
    structured: bool


@dataclass
class AuthenticationConfig:
    """Authentication configuration"""
    store_tokens: bool
    auto_update_check: bool
    prompt_on_first_sign: bool


@dataclass
class PerformanceConfig:
    """Performance configuration"""
    cache_keys: bool
    max_concurrent_signs: int
    default_timeout_secs: int
    default_max_retries: int


@dataclass
class DebugConfig:
    """Debug configuration"""
    enabled: bool
    log_canonical_strings: bool
    log_components: bool
    log_timing: bool


@dataclass
class EnvironmentConfig:
    """Environment-specific configuration"""
    signing: UnifiedSigningConfig
    verification: VerificationConfig
    logging: LoggingConfig
    authentication: AuthenticationConfig
    performance: PerformanceConfig


@dataclass
class UnifiedSecurityProfile:
    """Unified security profile"""
    description: str
    required_components: List[str]
    include_content_digest: bool
    digest_algorithm: str
    validate_nonces: bool
    allow_custom_nonces: bool


@dataclass
class DefaultConfig:
    """Default configuration values"""
    environment: str
    signing_mode: str
    output_format: str
    verbosity: int


@dataclass
class UnifiedConfig:
    """Unified configuration structure"""
    config_format_version: str
    environments: Dict[str, EnvironmentConfig]
    security_profiles: Dict[str, UnifiedSecurityProfile]
    defaults: DefaultConfig


class UnifiedConfigManager:
    """Unified configuration manager for Python SDK"""
    
    def __init__(self, config: UnifiedConfig, environment: Optional[str] = None):
        self.config = config
        self.current_environment = environment or config.defaults.environment
        self._validate()
    
    @classmethod
    def from_json(cls, json_string: str, environment: Optional[str] = None) -> 'UnifiedConfigManager':
        """Load unified configuration from JSON string"""
        try:
            data = json.loads(json_string)
            config = cls._parse_config_dict(data)
            return cls(config, environment)
        except json.JSONDecodeError as e:
            raise UnifiedConfigError(f"Failed to parse configuration JSON: {e}", "PARSE_ERROR")
        except (KeyError, TypeError, ValueError) as e:
            raise UnifiedConfigError(f"Invalid configuration format: {e}", "INVALID_FORMAT")
    
    @classmethod
    def from_file(cls, file_path: Union[str, Path], environment: Optional[str] = None) -> 'UnifiedConfigManager':
        """Load unified configuration from file"""
        try:
            path = Path(file_path)
            with open(path, 'r', encoding='utf-8') as f:
                json_string = f.read()
            return cls.from_json(json_string, environment)
        except OSError as e:
            raise UnifiedConfigError(f"Failed to read configuration file: {e}", "FILE_ERROR")
    
    @classmethod
    def from_url(cls, url: str, environment: Optional[str] = None) -> 'UnifiedConfigManager':
        """Load unified configuration from URL"""
        try:
            with urllib.request.urlopen(url) as response:
                json_string = response.read().decode('utf-8')
            return cls.from_json(json_string, environment)
        except urllib.error.URLError as e:
            raise UnifiedConfigError(f"Failed to fetch configuration from URL: {e}", "URL_ERROR")
    
    @classmethod
    def load_default(cls, environment: Optional[str] = None) -> 'UnifiedConfigManager':
        """Load default configuration"""
        # Try to load from common locations
        default_paths = [
            Path("config/unified-datafold-config.json"),
            Path("../config/unified-datafold-config.json"),
            Path("../../config/unified-datafold-config.json"),
        ]
        
        for path in default_paths:
            if path.exists():
                return cls.from_file(path, environment)
        
        raise UnifiedConfigError("Default configuration file not found", "FILE_NOT_FOUND")
    
    def set_environment(self, environment: str) -> None:
        """Set current environment"""
        if environment not in self.config.environments:
            raise UnifiedConfigError(f"Environment '{environment}' not found", "ENVIRONMENT_NOT_FOUND")
        self.current_environment = environment
    
    def get_current_environment_config(self) -> EnvironmentConfig:
        """Get current environment configuration"""
        env_config = self.config.environments.get(self.current_environment)
        if not env_config:
            raise UnifiedConfigError(f"Environment '{self.current_environment}' not found", "ENVIRONMENT_NOT_FOUND")
        return env_config
    
    def to_signing_config(self, key_id: str, private_key: bytes) -> SigningConfig:
        """Convert to Python SDK signing configuration"""
        env_config = self.get_current_environment_config()
        profile = self.get_security_profile(env_config.signing.policy)
        
        # Map required components to SignatureComponents format
        components = SignatureComponents(
            method='@method' in env_config.signing.required_components,
            target_uri='@target-uri' in env_config.signing.required_components,
            headers=[c for c in env_config.signing.required_components if not c.startswith('@')],
            content_digest=env_config.signing.include_content_digest
        )
        
        return SigningConfig(
            algorithm=SignatureAlgorithm.ED25519,
            key_id=key_id,
            private_key=private_key,
            components=components,
            nonce_generator=lambda: generate_nonce(),
            timestamp_generator=lambda: generate_timestamp()
        )
    
    def get_security_profile(self, name: str) -> UnifiedSecurityProfile:
        """Get security profile by name"""
        profile = self.config.security_profiles.get(name)
        if not profile:
            raise UnifiedConfigError(f"Security profile '{name}' not found", "PROFILE_NOT_FOUND")
        return profile
    
    def list_environments(self) -> List[str]:
        """List available environments"""
        return list(self.config.environments.keys())
    
    def list_security_profiles(self) -> List[str]:
        """List available security profiles"""
        return list(self.config.security_profiles.keys())
    
    def get_current_environment(self) -> str:
        """Get current environment name"""
        return self.current_environment
    
    def get_config(self) -> UnifiedConfig:
        """Get the full unified configuration"""
        return self.config
    
    def get_verification_config(self) -> VerificationConfig:
        """Get verification configuration for current environment"""
        return self.get_current_environment_config().verification
    
    def get_logging_config(self) -> LoggingConfig:
        """Get logging configuration for current environment"""
        return self.get_current_environment_config().logging
    
    def get_performance_config(self) -> PerformanceConfig:
        """Get performance configuration for current environment"""
        return self.get_current_environment_config().performance
    
    def _validate(self) -> None:
        """Validate the configuration"""
        # Validate default environment exists
        if self.config.defaults.environment not in self.config.environments:
            raise UnifiedConfigError(
                f"Default environment '{self.config.defaults.environment}' not found",
                "INVALID_DEFAULT_ENVIRONMENT"
            )
        
        # Validate each environment configuration
        for env_name, env_config in self.config.environments.items():
            # Validate signing policy references exist
            if env_config.signing.policy not in self.config.security_profiles:
                raise UnifiedConfigError(
                    f"Environment '{env_name}' references unknown security profile '{env_config.signing.policy}'",
                    "INVALID_SECURITY_PROFILE"
                )
            
            # Validate performance settings
            if env_config.performance.max_concurrent_signs <= 0:
                raise UnifiedConfigError(
                    f"Environment '{env_name}' has invalid max_concurrent_signs",
                    "INVALID_PERFORMANCE_CONFIG"
                )
            
            if env_config.signing.timeout_ms <= 0:
                raise UnifiedConfigError(
                    f"Environment '{env_name}' has invalid signing timeout",
                    "INVALID_SIGNING_CONFIG"
                )
    
    @staticmethod
    def _parse_config_dict(data: Dict[str, Any]) -> UnifiedConfig:
        """Parse configuration dictionary into structured objects"""
        
        # Parse environments
        environments = {}
        for env_name, env_data in data['environments'].items():
            debug_config = DebugConfig(**env_data['signing']['debug'])
            
            signing_config = UnifiedSigningConfig(
                policy=env_data['signing']['policy'],
                timeout_ms=env_data['signing']['timeout_ms'],
                required_components=env_data['signing']['required_components'],
                include_content_digest=env_data['signing']['include_content_digest'],
                include_timestamp=env_data['signing']['include_timestamp'],
                include_nonce=env_data['signing']['include_nonce'],
                max_body_size_mb=env_data['signing']['max_body_size_mb'],
                debug=debug_config
            )
            
            verification_config = VerificationConfig(**env_data['verification'])
            logging_config = LoggingConfig(**env_data['logging'])
            auth_config = AuthenticationConfig(**env_data['authentication'])
            perf_config = PerformanceConfig(**env_data['performance'])
            
            environments[env_name] = EnvironmentConfig(
                signing=signing_config,
                verification=verification_config,
                logging=logging_config,
                authentication=auth_config,
                performance=perf_config
            )
        
        # Parse security profiles
        security_profiles = {}
        for profile_name, profile_data in data['security_profiles'].items():
            security_profiles[profile_name] = UnifiedSecurityProfile(**profile_data)
        
        # Parse defaults
        defaults = DefaultConfig(**data['defaults'])
        
        return UnifiedConfig(
            config_format_version=data['config_format_version'],
            environments=environments,
            security_profiles=security_profiles,
            defaults=defaults
        )


def create_unified_config(config: UnifiedConfig, environment: Optional[str] = None) -> UnifiedConfigManager:
    """Create unified configuration manager from configuration object"""
    return UnifiedConfigManager(config, environment)


def load_unified_config_from_json(json_string: str, environment: Optional[str] = None) -> UnifiedConfigManager:
    """Load unified configuration from JSON string"""
    return UnifiedConfigManager.from_json(json_string, environment)


def load_unified_config_from_file(file_path: Union[str, Path], environment: Optional[str] = None) -> UnifiedConfigManager:
    """Load unified configuration from file"""
    return UnifiedConfigManager.from_file(file_path, environment)


def load_unified_config_from_url(url: str, environment: Optional[str] = None) -> UnifiedConfigManager:
    """Load unified configuration from URL"""
    return UnifiedConfigManager.from_url(url, environment)


def load_default_unified_config(environment: Optional[str] = None) -> UnifiedConfigManager:
    """Load default unified configuration"""
    return UnifiedConfigManager.load_default(environment)