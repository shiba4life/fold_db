"""
Verification Policy Constants for Python SDK
"""

import json
from pathlib import Path
from typing import Dict, Any, Optional, List
from dataclasses import dataclass

@dataclass
class VerificationPolicy:
    """Verification policy configuration"""
    name: str
    description: str
    verify_timestamp: bool
    max_timestamp_age: Optional[int]
    verify_nonce: bool
    verify_content_digest: bool
    required_components: List[str]
    allowed_algorithms: List[str]
    require_all_headers: bool

# Load shared policies once
_shared_policies: Optional[Dict[str, VerificationPolicy]] = None

def _get_shared_policies() -> Dict[str, VerificationPolicy]:
    """Load verification policies from shared configuration"""
    global _shared_policies
    
    if _shared_policies is not None:
        return _shared_policies
    
    config_path = Path(__file__).parent.parent.parent.parent.parent / 'config' / 'shared-policies.json'
    with open(config_path, 'r') as f:
        config = json.load(f)
    
    _shared_policies = {}
    for name, policy_data in config.items():
        _shared_policies[name] = VerificationPolicy(
            name=policy_data['name'],
            description=policy_data['description'],
            verify_timestamp=policy_data['verifyTimestamp'],
            max_timestamp_age=policy_data['maxTimestampAge'],
            verify_nonce=policy_data['verifyNonce'],
            verify_content_digest=policy_data['verifyContentDigest'],
            required_components=policy_data['requiredComponents'],
            allowed_algorithms=policy_data['allowedAlgorithms'],
            require_all_headers=policy_data['requireAllHeaders']
        )
    
    return _shared_policies

# Export standard policy constants
policies = _get_shared_policies()
STRICT: VerificationPolicy = policies['STRICT']
STANDARD: VerificationPolicy = policies['STANDARD']
LENIENT: VerificationPolicy = policies['LENIENT']
LEGACY: VerificationPolicy = policies['LEGACY']

def get_verification_policy(name: str) -> Optional[VerificationPolicy]:
    """Get verification policy by name"""
    policies = _get_shared_policies()
    return policies.get(name.upper())

def get_available_verification_policies() -> List[str]:
    """Get all available verification policy names"""
    policies = _get_shared_policies()
    return list(policies.keys())

# For backward compatibility
VERIFICATION_POLICIES = policies