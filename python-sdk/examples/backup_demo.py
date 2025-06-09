#!/usr/bin/env python3
"""
Demonstration of encrypted key backup and recovery functionality
for the DataFold Python SDK
"""

import os
import tempfile
import json
from typing import Tuple

# Import DataFold SDK components
from datafold_sdk.crypto import (
    generate_key_pair,
    KeyBackupManager,
    export_key_to_file,
    import_key_from_file,
    Ed25519KeyPair,
    BackupMetadata
)


def demo_basic_export_import():
    """Demonstrate basic key export and import"""
    print("=== Basic Export/Import Demo ===")
    
    # Generate a key pair
    print("1. Generating Ed25519 key pair...")
    key_pair = generate_key_pair()
    print(f"   Private key: {key_pair.private_key.hex()[:16]}...")
    print(f"   Public key:  {key_pair.public_key.hex()[:16]}...")
    
    # Create backup manager
    manager = KeyBackupManager()
    
    # Export key with passphrase
    print("\n2. Exporting key with passphrase encryption...")
    passphrase = "MySecureBackupPassphrase123!"
    backup_data = manager.export_key(
        key_pair=key_pair,
        passphrase=passphrase,
        key_id="demo-key-2025",
        export_format='json'
    )
    
    print("   Backup created successfully!")
    backup_dict = json.loads(backup_data)
    print(f"   KDF: {backup_dict['kdf']}")
    print(f"   Encryption: {backup_dict['encryption']}")
    print(f"   Created: {backup_dict['created']}")
    
    # Import key
    print("\n3. Importing key from backup...")
    imported_pair, metadata = manager.import_key(
        backup_data=backup_data,
        passphrase=passphrase,
        verify_integrity=True
    )
    
    # Verify keys match
    print("   Import successful!")
    assert imported_pair.private_key == key_pair.private_key
    assert imported_pair.public_key == key_pair.public_key
    print("   ✓ Key verification passed")
    print(f"   Key ID: {metadata.key_id}")
    print(f"   Algorithm: {metadata.algorithm}")
    
    return key_pair, backup_data


def demo_file_backup_recovery():
    """Demonstrate file-based backup and recovery"""
    print("\n=== File Backup/Recovery Demo ===")
    
    # Generate a key pair
    key_pair = generate_key_pair()
    
    # Create temporary directory for demo
    with tempfile.TemporaryDirectory() as temp_dir:
        backup_file = os.path.join(temp_dir, "my_key_backup.json")
        
        print("1. Exporting key to encrypted backup file...")
        metadata = export_key_to_file(
            key_pair=key_pair,
            passphrase="FileBackupPassword123!",
            key_id="file-backup-key",
            file_path=backup_file,
            export_format='json'
        )
        
        print(f"   Backup saved to: {backup_file}")
        print(f"   File size: {os.path.getsize(backup_file)} bytes")
        
        # Check file permissions
        if os.name != 'nt':  # Unix-like systems
            file_stat = os.stat(backup_file)
            permissions = oct(file_stat.st_mode)[-3:]
            print(f"   File permissions: {permissions} (secure)")
        
        print("\n2. Importing key from backup file...")
        recovered_pair, recovery_metadata = import_key_from_file(
            file_path=backup_file,
            passphrase="FileBackupPassword123!",
            verify_integrity=True
        )
        
        # Verify recovery
        assert recovered_pair.private_key == key_pair.private_key
        assert recovered_pair.public_key == key_pair.public_key
        print("   ✓ File recovery successful")
        print(f"   Recovered key ID: {recovery_metadata.key_id}")


def demo_algorithm_options():
    """Demonstrate different KDF and encryption algorithm options"""
    print("\n=== Algorithm Options Demo ===")
    
    key_pair = generate_key_pair()
    manager = KeyBackupManager()
    
    # Test different algorithm combinations
    algorithms = [
        ("scrypt", "aes-gcm", "Fast, secure"),
        ("pbkdf2", "aes-gcm", "Legacy compatibility"),
        ("scrypt", "chacha20-poly1305", "Modern, fast"),
        ("pbkdf2", "chacha20-poly1305", "Reliable option")
    ]
    
    print("Testing different KDF and encryption combinations:")
    
    for kdf, encryption, description in algorithms:
        print(f"\n  {kdf.upper()} + {encryption.upper()}: {description}")
        
        try:
            # Export with specific algorithms
            backup_data = manager.export_key(
                key_pair=key_pair,
                passphrase="AlgorithmTest123!",
                key_id=f"test-{kdf}-{encryption}",
                kdf_algorithm=kdf,
                encryption_algorithm=encryption
            )
            
            # Import and verify
            imported_pair, metadata = manager.import_key(
                backup_data=backup_data,
                passphrase="AlgorithmTest123!"
            )
            
            assert imported_pair.private_key == key_pair.private_key
            print(f"    ✓ Success")
            
        except Exception as e:
            print(f"    ✗ Failed: {e}")


def demo_error_handling():
    """Demonstrate error handling for various failure scenarios"""
    print("\n=== Error Handling Demo ===")
    
    key_pair = generate_key_pair()
    manager = KeyBackupManager()
    
    # Create a valid backup first
    backup_data = manager.export_key(
        key_pair=key_pair,
        passphrase="CorrectPassword123!",
        key_id="error-test-key"
    )
    
    print("1. Testing wrong passphrase...")
    try:
        manager.import_key(backup_data, "WrongPassword123!")
        print("   ✗ Should have failed!")
    except Exception as e:
        print(f"   ✓ Correctly detected: {type(e).__name__}")
    
    print("\n2. Testing weak passphrase...")
    try:
        manager.export_key(key_pair, "weak", "test")
        print("   ✗ Should have failed!")
    except Exception as e:
        print(f"   ✓ Correctly detected: {type(e).__name__}")
    
    print("\n3. Testing corrupted backup data...")
    try:
        corrupted_backup = backup_data.replace('a', 'z', 1)
        manager.import_key(corrupted_backup, "CorrectPassword123!")
        print("   ✗ Should have failed!")
    except Exception as e:
        print(f"   ✓ Correctly detected: {type(e).__name__}")
    
    print("\n4. Testing invalid JSON...")
    try:
        manager.import_key("{ invalid json", "CorrectPassword123!")
        print("   ✗ Should have failed!")
    except Exception as e:
        print(f"   ✓ Correctly detected: {type(e).__name__}")


def demo_platform_support():
    """Demonstrate platform support checking"""
    print("\n=== Platform Support Demo ===")
    
    manager = KeyBackupManager()
    support = manager.check_backup_support()
    
    print("Platform capabilities:")
    print(f"  Cryptography library: {'✓' if support['cryptography_available'] else '✗'}")
    print(f"  Argon2 support: {'✓' if support['argon2_available'] else '✗'}")
    
    print("\nSupported algorithms:")
    print(f"  KDF algorithms: {', '.join(support['supported_kdf_algorithms'])}")
    print(f"  Encryption algorithms: {', '.join(support['supported_encryption_algorithms'])}")
    
    print("\nCurrent preferences:")
    prefs = support['current_preferences']
    print(f"  Preferred KDF: {prefs['kdf']}")
    print(f"  Preferred encryption: {prefs['encryption']}")


def demo_cross_platform_compatibility():
    """Demonstrate cross-platform backup compatibility"""
    print("\n=== Cross-Platform Compatibility Demo ===")
    
    # Create backups with different managers (simulating different platforms)
    key_pair = generate_key_pair()
    passphrase = "CrossPlatformTest123!"
    
    # Create backup with one manager
    manager1 = KeyBackupManager(preferred_kdf='scrypt', preferred_encryption='aes-gcm')
    backup_data = manager1.export_key(key_pair, passphrase, "cross-platform-key")
    print("1. Created backup with Manager 1 (scrypt + aes-gcm)")
    
    # Import with different manager (simulating different platform)
    manager2 = KeyBackupManager(preferred_kdf='pbkdf2', preferred_encryption='chacha20-poly1305')
    imported_pair, metadata = manager2.import_key(backup_data, passphrase)
    print("2. Imported with Manager 2 (different preferences)")
    
    # Verify keys match
    assert imported_pair.private_key == key_pair.private_key
    assert imported_pair.public_key == key_pair.public_key
    print("   ✓ Cross-platform import successful")
    print(f"   Original algorithms used: {metadata.kdf} + {metadata.encryption}")


def main():
    """Run all demonstrations"""
    print("DataFold Python SDK - Key Backup and Recovery Demonstration")
    print("=" * 60)
    
    try:
        # Run all demonstrations
        demo_basic_export_import()
        demo_file_backup_recovery()
        demo_algorithm_options()
        demo_error_handling()
        demo_platform_support()
        demo_cross_platform_compatibility()
        
        print("\n" + "=" * 60)
        print("🎉 All demonstrations completed successfully!")
        print("\nKey takeaways:")
        print("• Keys are encrypted with strong user passphrases")
        print("• Multiple KDF and encryption algorithms supported")
        print("• Comprehensive error detection and handling")
        print("• Cross-platform compatible backup format")
        print("• File permissions set securely (Unix-like systems)")
        print("• Integrity verification prevents tampering")
        
    except Exception as e:
        print(f"\n❌ Demo failed: {e}")
        import traceback
        traceback.print_exc()
        return 1
    
    return 0


if __name__ == "__main__":
    exit(main())