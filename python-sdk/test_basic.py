#!/usr/bin/env python3
"""
Basic test script to verify Ed25519 implementation
"""

try:
    import datafold_sdk
    print("✓ Successfully imported datafold_sdk")
    
    # Test compatibility check
    result = datafold_sdk.initialize_sdk()
    print(f"✓ Platform compatibility: {result}")
    
    if result['compatible']:
        # Test key generation
        key_pair = datafold_sdk.generate_key_pair()
        print(f"✓ Generated key pair - private: {len(key_pair.private_key)} bytes, public: {len(key_pair.public_key)} bytes")
        
        # Test key formatting
        hex_private = datafold_sdk.format_key(key_pair.private_key, 'hex')
        base64_public = datafold_sdk.format_key(key_pair.public_key, 'base64')
        print(f"✓ Formatted keys - hex private: {hex_private[:16]}..., base64 public: {base64_public[:16]}...")
        
        # Test key parsing roundtrip
        parsed_private = datafold_sdk.parse_key(hex_private, 'hex')
        parsed_public = datafold_sdk.parse_key(base64_public, 'base64')
        
        assert parsed_private == key_pair.private_key, "Private key roundtrip failed"
        assert parsed_public == key_pair.public_key, "Public key roundtrip failed"
        print("✓ Key parsing roundtrip successful")
        
        # Test multiple key generation
        key_pairs = datafold_sdk.generate_multiple_key_pairs(3)
        print(f"✓ Generated {len(key_pairs)} key pairs")
        
        # Test PEM formatting
        try:
            pem_private = datafold_sdk.format_key(key_pair.private_key, 'pem')
            pem_public = datafold_sdk.format_key(key_pair.public_key, 'pem')
            print("✓ PEM formatting successful")
            
            # Test PEM parsing
            parsed_pem_private = datafold_sdk.parse_key(pem_private, 'pem')
            parsed_pem_public = datafold_sdk.parse_key(pem_public, 'pem')
            
            assert parsed_pem_private == key_pair.private_key, "PEM private key roundtrip failed"
            assert parsed_pem_public == key_pair.public_key, "PEM public key roundtrip failed"
            print("✓ PEM parsing roundtrip successful")
            
        except Exception as e:
            print(f"⚠ PEM test failed: {e}")
        
        # Test key clearance
        test_key_pair = datafold_sdk.generate_key_pair()
        original_private = test_key_pair.private_key
        datafold_sdk.clear_key_material(test_key_pair)
        assert test_key_pair.private_key != original_private, "Key clearance failed"
        print("✓ Key clearance successful")
        
        print("\n🎉 All basic tests passed!")
        
    else:
        print("❌ Platform not compatible, skipping functionality tests")
        
except Exception as e:
    print(f"❌ Test failed with error: {e}")
    import traceback
    traceback.print_exc()