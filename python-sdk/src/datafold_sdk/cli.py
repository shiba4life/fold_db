"""
Command-line interface for DataFold Python SDK
Provides comprehensive key management and server integration functionality
"""

import argparse
import sys
import json
import uuid
from typing import Optional, Dict, Any

from . import generate_key_pair, format_key, initialize_sdk, __version__
from .crypto.storage import get_default_storage
from .integration import DataFoldClient, quick_setup, register_and_verify_workflow
from .exceptions import DataFoldSDKError, ServerCommunicationError, StorageError


def create_parser() -> argparse.ArgumentParser:
    """Create the main argument parser with subcommands."""
    parser = argparse.ArgumentParser(
        prog='datafold-cli',
        description='DataFold SDK command-line interface for key management and server integration'
    )
    
    parser.add_argument(
        '--version',
        action='version',
        version=f'DataFold Python SDK {__version__}'
    )
    
    parser.add_argument(
        '--check-compatibility',
        action='store_true',
        help='Check platform compatibility and exit'
    )
    
    # Create subparsers for different commands
    subparsers = parser.add_subparsers(dest='command', help='Available commands')
    
    # Key generation subcommand
    setup_keygen_parser(subparsers)
    
    # Server integration subcommands
    setup_server_parser(subparsers)
    
    # Storage management subcommands
    setup_storage_parser(subparsers)
    
    return parser


def setup_keygen_parser(subparsers):
    """Setup key generation subcommand."""
    keygen_parser = subparsers.add_parser('keygen', help='Generate Ed25519 key pairs')
    keygen_parser.add_argument(
        '--format',
        choices=['hex', 'base64', 'pem', 'bytes'],
        default='hex',
        help='Output format for keys (default: hex)'
    )
    keygen_parser.add_argument(
        '--count',
        type=int,
        default=1,
        help='Number of key pairs to generate (default: 1)'
    )
    keygen_parser.add_argument(
        '--no-validation',
        action='store_true',
        help='Skip key validation for faster generation'
    )
    keygen_parser.add_argument(
        '--private-only',
        action='store_true',
        help='Output only private keys'
    )
    keygen_parser.add_argument(
        '--public-only',
        action='store_true',
        help='Output only public keys'
    )
    keygen_parser.add_argument(
        '--save-to-storage',
        help='Save generated key pair to storage with given name'
    )


def setup_server_parser(subparsers):
    """Setup server integration subcommands."""
    server_parser = subparsers.add_parser('server', help='Server integration commands')
    server_subparsers = server_parser.add_subparsers(dest='server_command', help='Server operations')
    
    # Register key with server
    register_parser = server_subparsers.add_parser('register', help='Register public key with server')
    register_parser.add_argument('--server-url', required=True, help='DataFold server URL')
    register_parser.add_argument('--client-id', help='Client identifier (auto-generated if not provided)')
    register_parser.add_argument('--user-id', help='User identifier')
    register_parser.add_argument('--key-name', help='Human-readable key name')
    register_parser.add_argument('--storage-key', help='Storage key name for saved key pair')
    register_parser.add_argument('--private-key-hex', help='Private key in hex format')
    register_parser.add_argument('--save-session', action='store_true', help='Save session to local storage')
    register_parser.add_argument('--metadata', help='JSON metadata to include with registration')
    
    # Verify signature with server
    verify_parser = server_subparsers.add_parser('verify', help='Verify signature with server')
    verify_parser.add_argument('--server-url', required=True, help='DataFold server URL')
    verify_parser.add_argument('--client-id', required=True, help='Client identifier')
    verify_parser.add_argument('--message', required=True, help='Message to verify')
    verify_parser.add_argument('--signature', required=True, help='Signature to verify (hex)')
    verify_parser.add_argument('--message-encoding', choices=['utf8', 'hex', 'base64'], default='utf8', help='Message encoding')
    
    # Sign message and verify with server
    sign_verify_parser = server_subparsers.add_parser('sign-verify', help='Sign message and verify with server')
    sign_verify_parser.add_argument('--server-url', required=True, help='DataFold server URL')
    sign_verify_parser.add_argument('--client-id', help='Client identifier (auto-generated if not provided)')
    sign_verify_parser.add_argument('--message', required=True, help='Message to sign and verify')
    sign_verify_parser.add_argument('--storage-key', help='Storage key name for saved key pair')
    sign_verify_parser.add_argument('--private-key-hex', help='Private key in hex format')
    sign_verify_parser.add_argument('--message-encoding', choices=['utf8', 'hex', 'base64'], default='utf8', help='Message encoding')
    
    # Test complete workflow
    test_parser = server_subparsers.add_parser('test', help='Test complete registration and verification workflow')
    test_parser.add_argument('--server-url', required=True, help='DataFold server URL')
    test_parser.add_argument('--client-id', help='Client identifier (auto-generated if not provided)')
    test_parser.add_argument('--test-message', default='DataFold CLI Test Message', help='Test message to sign and verify')
    
    # Get registration status
    status_parser = server_subparsers.add_parser('status', help='Get registration status for client')
    status_parser.add_argument('--server-url', required=True, help='DataFold server URL')
    status_parser.add_argument('--client-id', required=True, help='Client identifier')


def setup_storage_parser(subparsers):
    """Setup storage management subcommands."""
    storage_parser = subparsers.add_parser('storage', help='Key storage management')
    storage_subparsers = storage_parser.add_subparsers(dest='storage_command', help='Storage operations')
    
    # List stored keys
    list_parser = storage_subparsers.add_parser('list', help='List stored key pairs')
    
    # Load key from storage
    load_parser = storage_subparsers.add_parser('load', help='Load key pair from storage')
    load_parser.add_argument('storage_key', help='Storage key name')
    load_parser.add_argument('--format', choices=['hex', 'base64', 'pem'], default='hex', help='Output format')
    load_parser.add_argument('--private-only', action='store_true', help='Output only private key')
    load_parser.add_argument('--public-only', action='store_true', help='Output only public key')
    
    # Delete key from storage
    delete_parser = storage_subparsers.add_parser('delete', help='Delete key pair from storage')
    delete_parser.add_argument('storage_key', help='Storage key name')
    delete_parser.add_argument('--confirm', action='store_true', help='Skip confirmation prompt')


def handle_keygen_command(args) -> int:
    """Handle key generation command."""
    try:
        # Validate arguments
        if args.count < 1 or args.count > 100:
            print("Error: Count must be between 1 and 100", file=sys.stderr)
            return 1
        
        if args.private_only and args.public_only:
            print("Error: Cannot specify both --private-only and --public-only", file=sys.stderr)
            return 1
        
        # Generate key pairs
        validate = not args.no_validation
        
        if args.count == 1:
            key_pair = generate_key_pair(validate=validate)
            key_pairs = [key_pair]
        else:
            from . import generate_multiple_key_pairs
            key_pairs = generate_multiple_key_pairs(args.count, validate=validate)
        
        # Save to storage if requested
        storage = None
        if args.save_to_storage:
            storage = get_default_storage()
        
        # Output keys
        for i, key_pair in enumerate(key_pairs):
            if args.count > 1:
                print(f"# Key pair {i + 1}")
            
            if not args.public_only:
                private_formatted = format_key(key_pair.private_key, args.format)
                if args.format == 'pem':
                    print("Private Key:")
                    print(private_formatted)
                elif args.format == 'bytes':
                    print(f"Private Key: {private_formatted!r}")
                else:
                    print(f"Private Key: {private_formatted}")
            
            if not args.private_only:
                public_formatted = format_key(key_pair.public_key, args.format)
                if args.format == 'pem':
                    print("Public Key:")
                    print(public_formatted)
                elif args.format == 'bytes':
                    print(f"Public Key: {public_formatted!r}")
                else:
                    print(f"Public Key: {public_formatted}")
            
            # Save to storage if requested
            if storage and args.save_to_storage:
                storage_key = args.save_to_storage
                if args.count > 1:
                    storage_key = f"{args.save_to_storage}_{i + 1}"
                storage.store_key(storage_key, key_pair)
                print(f"Key pair saved to storage as: {storage_key}")
            
            if args.count > 1 and i < len(key_pairs) - 1:
                print()  # Blank line between key pairs
        
        return 0
        
    except Exception as e:
        print(f"Error generating keys: {e}", file=sys.stderr)
        return 1


def handle_server_command(args) -> int:
    """Handle server integration commands."""
    try:
        if args.server_command == 'register':
            return handle_register_command(args)
        elif args.server_command == 'verify':
            return handle_verify_command(args)
        elif args.server_command == 'sign-verify':
            return handle_sign_verify_command(args)
        elif args.server_command == 'test':
            return handle_test_command(args)
        elif args.server_command == 'status':
            return handle_status_command(args)
        else:
            print("Error: No server subcommand specified", file=sys.stderr)
            return 1
    except ServerCommunicationError as e:
        print(f"Server communication error: {e}", file=sys.stderr)
        return 1
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1


def handle_register_command(args) -> int:
    """Handle key registration with server."""
    client = DataFoldClient(args.server_url)
    
    # Get key pair
    if args.storage_key:
        storage = get_default_storage()
        key_pair = storage.retrieve_key(args.storage_key)
        print(f"Loaded key pair from storage: {args.storage_key}")
    elif args.private_key_hex:
        from .crypto.ed25519 import Ed25519KeyPair
        private_key_bytes = bytes.fromhex(args.private_key_hex)
        key_pair = Ed25519KeyPair.from_private_key(private_key_bytes)
        print("Using provided private key")
    else:
        key_pair = generate_key_pair()
        print("Generated new key pair")
    
    # Parse metadata if provided
    metadata = None
    if args.metadata:
        metadata = json.loads(args.metadata)
    
    # Generate client ID if not provided
    client_id = args.client_id or f"cli_client_{uuid.uuid4().hex[:8]}"
    
    # Register with server
    session = client.create_new_session(
        client_id=client_id,
        user_id=args.user_id,
        key_name=args.key_name,
        metadata=metadata,
        auto_register=True,
        save_to_storage=args.save_session
    )
    
    print(f"✓ Registration successful!")
    print(f"  Registration ID: {session.registration.registration_id}")
    print(f"  Client ID: {session.registration.client_id}")
    print(f"  Public Key: {session.registration.public_key}")
    print(f"  Status: {session.registration.status}")
    print(f"  Registered At: {session.registration.registered_at}")
    
    if args.save_session:
        print(f"  Session saved to storage")
    
    client.close()
    return 0


def handle_verify_command(args) -> int:
    """Handle signature verification with server."""
    client = DataFoldClient(args.server_url)
    
    # Convert signature from hex
    signature_bytes = bytes.fromhex(args.signature)
    
    # Verify signature
    result = client.verify_signature(
        args.client_id,
        args.message,
        signature_bytes,
        args.message_encoding
    )
    
    print(f"Signature verification: {'✓ VERIFIED' if result.verified else '✗ FAILED'}")
    print(f"  Client ID: {result.client_id}")
    print(f"  Public Key: {result.public_key}")
    print(f"  Message Hash: {result.message_hash}")
    print(f"  Verified At: {result.verified_at}")
    
    client.close()
    return 0 if result.verified else 1


def handle_sign_verify_command(args) -> int:
    """Handle message signing and verification with server."""
    client = DataFoldClient(args.server_url)
    
    # Get key pair
    if args.storage_key:
        storage = get_default_storage()
        key_pair = storage.retrieve_key(args.storage_key)
        print(f"Loaded key pair from storage: {args.storage_key}")
    elif args.private_key_hex:
        from .crypto.ed25519 import Ed25519KeyPair
        private_key_bytes = bytes.fromhex(args.private_key_hex)
        key_pair = Ed25519KeyPair.from_private_key(private_key_bytes)
        print("Using provided private key")
    else:
        print("Error: Must provide either --storage-key or --private-key-hex", file=sys.stderr)
        return 1
    
    # Generate client ID if not provided
    client_id = args.client_id or f"cli_client_{uuid.uuid4().hex[:8]}"
    
    # Create session with existing key
    session = client.load_session_from_storage(args.storage_key, client_id) if args.storage_key else None
    if not session:
        # Create temporary session
        from .integration import ClientSession
        session = ClientSession(
            key_pair=key_pair,
            client_id=client_id,
            http_client=client.http_client
        )
    
    # Sign message
    from .crypto.ed25519 import sign_message
    signature = sign_message(key_pair.private_key, args.message)
    
    print(f"✓ Message signed")
    print(f"  Message: {args.message}")
    print(f"  Signature: {signature.hex()}")
    
    # Verify with server
    result = session.verify_with_server(args.message, signature, args.message_encoding)
    
    print(f"✓ Server verification: {'VERIFIED' if result.verified else 'FAILED'}")
    print(f"  Client ID: {result.client_id}")
    print(f"  Public Key: {result.public_key}")
    print(f"  Verified At: {result.verified_at}")
    
    client.close()
    return 0 if result.verified else 1


def handle_test_command(args) -> int:
    """Handle complete workflow test."""
    print(f"Running complete workflow test with server: {args.server_url}")
    
    client_id = args.client_id or f"test_cli_{uuid.uuid4().hex[:8]}"
    session, result = register_and_verify_workflow(
        args.server_url,
        args.test_message,
        client_id
    )
    
    print(f"✓ Complete workflow test successful!")
    print(f"  Client ID: {session.client_id}")
    print(f"  Registration ID: {session.registration.registration_id}")
    print(f"  Test Message: {args.test_message}")
    print(f"  Signature Verified: {result.verified}")
    print(f"  Message Hash: {result.message_hash}")
    
    session.close()
    return 0


def handle_status_command(args) -> int:
    """Handle registration status check."""
    client = DataFoldClient(args.server_url)
    
    try:
        registration = client.http_client.get_key_status(args.client_id)
        
        print(f"Registration Status for {args.client_id}:")
        print(f"  Registration ID: {registration.registration_id}")
        print(f"  Public Key: {registration.public_key}")
        print(f"  Key Name: {registration.key_name or 'N/A'}")
        print(f"  Status: {registration.status}")
        print(f"  Registered At: {registration.registered_at}")
        
        client.close()
        return 0
        
    except ServerCommunicationError as e:
        if "CLIENT_NOT_FOUND" in str(e):
            print(f"Client {args.client_id} is not registered", file=sys.stderr)
        else:
            print(f"Error checking status: {e}", file=sys.stderr)
        client.close()
        return 1


def handle_storage_command(args) -> int:
    """Handle storage management commands."""
    try:
        if args.storage_command == 'list':
            return handle_storage_list_command(args)
        elif args.storage_command == 'load':
            return handle_storage_load_command(args)
        elif args.storage_command == 'delete':
            return handle_storage_delete_command(args)
        else:
            print("Error: No storage subcommand specified", file=sys.stderr)
            return 1
    except StorageError as e:
        print(f"Storage error: {e}", file=sys.stderr)
        return 1
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1


def handle_storage_list_command(args) -> int:
    """Handle listing stored keys."""
    client = DataFoldClient("http://localhost:9001")  # URL not used for storage operations
    keys = client.list_stored_keys()
    
    if not keys:
        print("No keys stored")
    else:
        print("Stored keys:")
        for key in keys:
            print(f"  {key}")
    
    return 0


def handle_storage_load_command(args) -> int:
    """Handle loading key from storage."""
    storage = get_default_storage()
    key_pair = storage.retrieve_key(args.storage_key)
    
    print(f"Key pair loaded from storage: {args.storage_key}")
    
    if not args.public_only:
        private_formatted = format_key(key_pair.private_key, args.format)
        if args.format == 'pem':
            print("Private Key:")
            print(private_formatted)
        else:
            print(f"Private Key: {private_formatted}")
    
    if not args.private_only:
        public_formatted = format_key(key_pair.public_key, args.format)
        if args.format == 'pem':
            print("Public Key:")
            print(public_formatted)
        else:
            print(f"Public Key: {public_formatted}")
    
    return 0


def handle_storage_delete_command(args) -> int:
    """Handle deleting key from storage."""
    if not args.confirm:
        response = input(f"Are you sure you want to delete key '{args.storage_key}'? (y/N): ")
        if response.lower() not in ['y', 'yes']:
            print("Operation cancelled")
            return 0
    
    storage = get_default_storage()
    # Note: SecureKeyStorage doesn't have a delete method yet, but we can simulate it
    print(f"Key '{args.storage_key}' would be deleted (delete functionality not yet implemented)")
    return 0


def main(argv: Optional[list] = None) -> int:
    """
    Main entry point for the CLI
    
    Args:
        argv: Command line arguments (None to use sys.argv)
        
    Returns:
        int: Exit code (0 for success, non-zero for failure)
    """
    if argv is None:
        argv = sys.argv[1:]
    
    parser = create_parser()
    args = parser.parse_args(argv)
    
    try:
        # Check compatibility if requested
        if args.check_compatibility:
            result = initialize_sdk()
            if result['compatible']:
                print("✓ Platform is compatible with DataFold SDK")
                for warning in result['warnings']:
                    print(f"  Warning: {warning}")
                return 0
            else:
                print("✗ Platform is not compatible with DataFold SDK")
                for warning in result['warnings']:
                    print(f"  Error: {warning}")
                return 1
        
        # Initialize SDK for all operations
        result = initialize_sdk()
        if not result['compatible']:
            print("Error: Platform not compatible with DataFold SDK", file=sys.stderr)
            for warning in result['warnings']:
                print(f"  {warning}", file=sys.stderr)
            return 1
        
        # Handle commands
        if args.command == 'keygen':
            return handle_keygen_command(args)
        elif args.command == 'server':
            return handle_server_command(args)
        elif args.command == 'storage':
            return handle_storage_command(args)
        else:
            # No command specified, show help
            parser.print_help()
            return 1
        
    except KeyboardInterrupt:
        print("\nOperation cancelled by user", file=sys.stderr)
        return 130
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1


if __name__ == '__main__':
    sys.exit(main())