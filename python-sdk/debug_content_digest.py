#!/usr/bin/env python3

from datafold_sdk.signing import create_from_profile, RFC9421Signer, SignableRequest, HttpMethod
from datafold_sdk.signing.canonical_message import extract_covered_components
from datafold_sdk.crypto.ed25519 import generate_key_pair

# Generate key pair
key_pair = generate_key_pair()

# Create config using standard profile
config = create_from_profile("standard", "test-client", key_pair.private_key)

print("Config components:")
print(f"  content_digest: {config.components.content_digest}")
print(f"  headers: {config.components.headers}")

# Create signer
signer = RFC9421Signer(config)

# Create request with body
request = SignableRequest(
    method=HttpMethod.POST,
    url="https://api.datafold.com/api/crypto/keys/register",
    headers={"content-type": "application/json"},
    body='{"client_id": "test", "public_key": "abc123"}'
)

print(f"\nRequest body: {request.body}")
print(f"Has body: {signer._has_request_body(request)}")

# Sign request
result = signer.sign_request(request)

print(f"\nResult headers: {list(result.headers.keys())}")
print(f"Content-digest present: {'content-digest' in result.headers}")

if 'content-digest' in result.headers:
    print(f"Content-digest value: {result.headers['content-digest']}")

print(f"\nCanonical message:")
print(result.canonical_message)

print(f"\nCovered components:")
covered = extract_covered_components(result.canonical_message)
print(covered)

print(f"\nSignature input:")
print(result.signature_input)