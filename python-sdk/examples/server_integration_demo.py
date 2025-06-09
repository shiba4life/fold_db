#!/usr/bin/env python3
"""
DataFold SDK Server Integration Demo

This example demonstrates the complete workflow of integrating with a DataFold
server, including key generation, registration, and signature verification.
"""

import sys
import json
import logging
from typing import Optional

# Import DataFold SDK components
from datafold_sdk import (
    DataFoldClient, quick_setup, register_and_verify_workflow,
    generate_key_pair, sign_message, verify_signature,
    ServerCommunicationError, ValidationError
)

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


def demo_basic_workflow(server_url: str):
    """Demonstrate basic workflow with quick setup."""
    print("\n=== Basic Workflow Demo ===")
    
    try:
        # Quick setup creates key pair and registers with server
        session = quick_setup(
            server_url=server_url,
            client_id="demo-basic-client",
            key_name="demo-key",
            metadata={"demo": "basic_workflow"}
        )
        
        print(f"✅ Client registered: {session.client_id}")
        if session.registration:
            print(f"✅ Registration ID: {session.registration.registration_id}")
            print(f"✅ Status: {session.registration.status}")
        
        # Sign and verify a message
        message = "Hello from DataFold SDK!"
        signature = session.sign_message(message)
        
        print(f"✅ Message signed: {len(signature)} bytes")
        print(f"   Signature: {signature.hex()[:32]}...")
        
        # Verify with server
        result = session.verify_with_server(message, signature)
        print(f"✅ Server verification: {result.verified}")
        print(f"   Message hash: {result.message_hash[:16]}...")
        
        return True
        
    except Exception as e:
        print(f"❌ Basic workflow failed: {e}")
        return False


def demo_advanced_workflow(server_url: str):
    """Demonstrate advanced workflow with manual steps."""
    print("\n=== Advanced Workflow Demo ===")
    
    try:
        # Create client with custom configuration
        client = DataFoldClient(
            server_url=server_url,
            timeout=30.0,
            verify_ssl=False,  # For demo with local server
            retry_attempts=2
        )
        
        print("✅ DataFold client created")
        
        # Create session with detailed options
        session = client.create_new_session(
            client_id="demo-advanced-client",
            user_id="demo-user-123",
            key_name="advanced-demo-key",
            metadata={
                "application": "DataFold SDK Demo",
                "version": "1.0.0",
                "environment": "development"
            },
            auto_register=True,
            save_to_storage=False  # Don't persist for demo
        )
        
        print(f"✅ Advanced session created: {session.client_id}")
        
        # Check registration details
        if session.registration:
            print(f"✅ Registration details:")
            print(f"   ID: {session.registration.registration_id}")
            print(f"   Public Key: {session.registration.public_key[:32]}...")
            print(f"   Registered: {session.registration.registered_at}")
        
        # Demonstrate multiple message signing
        messages = [
            "First message",
            "Second message with more content",
            "{'data': 'json-like content', 'timestamp': 1234567890}"
        ]
        
        for i, msg in enumerate(messages, 1):
            signature = session.sign_message(msg)
            result = session.verify_with_server(msg, signature)
            
            print(f"✅ Message {i} verification: {result.verified}")
            
            if not result.verified:
                print(f"❌ Verification failed for message {i}")
                return False
        
        # Check key status
        status = client.http_client.get_key_status(session.client_id)
        print(f"✅ Key status check: {status.status}")
        
        return True
        
    except Exception as e:
        print(f"❌ Advanced workflow failed: {e}")
        return False


def demo_error_handling(server_url: str):
    """Demonstrate error handling scenarios."""
    print("\n=== Error Handling Demo ===")
    
    try:
        client = DataFoldClient(server_url)
        
        # Test 1: Invalid signature verification
        print("Test 1: Invalid signature verification")
        try:
            result = client.verify_signature(
                client_id="nonexistent-client",
                message="test message",
                signature=b'x' * 64  # Invalid signature
            )
            print(f"   Result: {result.verified} (expected: False)")
        except ServerCommunicationError as e:
            print(f"   Server error (expected): {e}")
        
        # Test 2: Invalid input validation
        print("Test 2: Input validation")
        try:
            client.verify_signature("", "message", b'short')  # Invalid inputs
        except ValidationError as e:
            print(f"   Validation error (expected): {e}")
        
        # Test 3: Network error simulation
        print("Test 3: Network error handling")
        try:
            # Create client with invalid URL to simulate network error
            bad_client = DataFoldClient("http://nonexistent-server:9999")
            bad_client.create_new_session(client_id="test")
        except ServerCommunicationError as e:
            print(f"   Network error (expected): Connection error")
        except Exception as e:
            print(f"   Other error: {e}")
        
        print("✅ Error handling tests completed")
        return True
        
    except Exception as e:
        print(f"❌ Error handling demo failed: {e}")
        return False


def demo_local_operations():
    """Demonstrate local key operations without server."""
    print("\n=== Local Operations Demo ===")
    
    try:
        # Generate key pair locally
        key_pair = generate_key_pair()
        print(f"✅ Key pair generated")
        print(f"   Private key: {len(key_pair.private_key)} bytes")
        print(f"   Public key: {len(key_pair.public_key)} bytes")
        
        # Local message signing
        message = "Local signature test"
        signature = sign_message(key_pair.private_key, message)
        print(f"✅ Message signed locally: {len(signature)} bytes")
        
        # Local signature verification
        is_valid = verify_signature(key_pair.public_key, message, signature)
        print(f"✅ Local verification: {is_valid}")
        
        # Test with modified message (should fail)
        modified_message = message + " modified"
        is_valid_modified = verify_signature(key_pair.public_key, modified_message, signature)
        print(f"✅ Modified message verification: {is_valid_modified} (expected: False)")
        
        return True
        
    except Exception as e:
        print(f"❌ Local operations demo failed: {e}")
        return False


def demo_complete_workflow(server_url: str):
    """Demonstrate the complete end-to-end workflow."""
    print("\n=== Complete Workflow Demo ===")
    
    try:
        # Use the convenience function for complete workflow
        session, result = register_and_verify_workflow(
            server_url=server_url,
            message="Complete workflow test message",
            client_id="demo-complete-client"
        )
        
        print(f"✅ Complete workflow successful")
        print(f"   Client ID: {session.client_id}")
        print(f"   Registration ID: {session.registration.registration_id}")
        print(f"   Verification result: {result.verified}")
        print(f"   Message hash: {result.message_hash}")
        
        return True
        
    except Exception as e:
        print(f"❌ Complete workflow failed: {e}")
        return False


def main():
    """Main demo function."""
    print("DataFold SDK Server Integration Demo")
    print("====================================")
    
    # Default server URL (can be overridden via command line)
    server_url = "http://localhost:9001"
    
    if len(sys.argv) > 1:
        server_url = sys.argv[1]
    
    print(f"Using DataFold server: {server_url}")
    print("Note: Make sure the DataFold server is running before running this demo")
    
    # Run all demos
    demos = [
        ("Local Operations", lambda: demo_local_operations()),
        ("Basic Workflow", lambda: demo_basic_workflow(server_url)),
        ("Advanced Workflow", lambda: demo_advanced_workflow(server_url)),
        ("Error Handling", lambda: demo_error_handling(server_url)),
        ("Complete Workflow", lambda: demo_complete_workflow(server_url)),
    ]
    
    results = {}
    
    for demo_name, demo_func in demos:
        try:
            print(f"\n{'='*60}")
            results[demo_name] = demo_func()
        except KeyboardInterrupt:
            print(f"\n❌ Demo interrupted by user")
            break
        except Exception as e:
            print(f"❌ Demo '{demo_name}' failed with unexpected error: {e}")
            results[demo_name] = False
    
    # Print summary
    print(f"\n{'='*60}")
    print("Demo Results Summary:")
    print("=" * 20)
    
    for demo_name, success in results.items():
        status = "✅ PASS" if success else "❌ FAIL"
        print(f"{demo_name:<20} {status}")
    
    total_demos = len(results)
    passed_demos = sum(results.values())
    
    print(f"\nOverall: {passed_demos}/{total_demos} demos passed")
    
    if passed_demos == total_demos:
        print("🎉 All demos completed successfully!")
        return 0
    else:
        print("⚠️  Some demos failed. Check server connectivity and configuration.")
        return 1


if __name__ == "__main__":
    exit_code = main()
    sys.exit(exit_code)