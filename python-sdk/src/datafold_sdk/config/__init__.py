"""
Configuration management for DataFold Python SDK

This module provides unified configuration management compatible with
the DataFold unified configuration format across all platforms.
"""

from .unified_config import (
    UnifiedConfig,
    UnifiedConfigManager,
    EnvironmentConfig,
    UnifiedSigningConfig,
    VerificationConfig,
    LoggingConfig,
    AuthenticationConfig,
    PerformanceConfig,
    DebugConfig,
    UnifiedSecurityProfile,
    DefaultConfig,
    UnifiedConfigError,
    create_unified_config,
    load_unified_config_from_json,
    load_unified_config_from_file,
    load_unified_config_from_url,
    load_default_unified_config,
)

__all__ = [
    'UnifiedConfig',
    'UnifiedConfigManager',
    'EnvironmentConfig',
    'UnifiedSigningConfig',
    'VerificationConfig',
    'LoggingConfig',
    'AuthenticationConfig',
    'PerformanceConfig',
    'DebugConfig',
    'UnifiedSecurityProfile',
    'DefaultConfig',
    'UnifiedConfigError',
    'create_unified_config',
    'load_unified_config_from_json',
    'load_unified_config_from_file',
    'load_unified_config_from_url',
    'load_default_unified_config',
]