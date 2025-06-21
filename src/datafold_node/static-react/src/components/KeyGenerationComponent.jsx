// React component for Ed25519 key generation

import { useState } from 'react';
import { KeyIcon, ClipboardIcon, CheckIcon, ExclamationTriangleIcon, ShieldCheckIcon } from '@heroicons/react/24/outline';
import { useSecurityAPI } from '../hooks/useSecurityAPI';
import { useKeyLifecycle } from '../hooks/useKeyLifecycle';

// Ed25519 utilities using @noble/ed25519
const bytesToHex = (bytes) => {
  return Array.from(bytes)
    .map(b => b.toString(16).padStart(2, '0'))
    .join('');
};

const generateKeyPairWithHex = async () => {
  try {
    // Generate a secure random private key
    const privateKey = ed.utils.randomPrivateKey();
    
    // Derive the public key from the private key
    const publicKey = await ed.getPublicKeyAsync(privateKey);
    
    const keyPair = {
      privateKey,
      publicKey
    };
    
    return {
      keyPair,
      publicKeyHex: bytesToHex(publicKey),
      privateKeyHex: bytesToHex(privateKey)
    };
  } catch (error) {
    throw new Error(`Failed to generate Ed25519 keypair: ${error.message}`);
  }
};

const KeyGenerationComponent = ({
  keyPair,
  publicKeyBase64,
  isRegistered,
  setIsRegistered,
  error,
  setError,
  generateKeys,
  clearKeys,
}) => {
  const [isRegistering, setIsRegistering] = useState(false);
  const [copiedField, setCopiedField] = useState(null);
  useKeyLifecycle(clearKeys);

  const {
    result: verificationResult,
    error: verificationError,
    isLoading: isVerifying,
    testSignatureVerification,
  } = useSecurityAPI(keyPair, publicKeyBase64);

  const registerPublicKey = async () => {
    if (!publicKeyBase64) return;

    setIsRegistering(true);
    setError(null);

    try {
      const requestBody = {
        public_key: publicKeyBase64,
        owner_id: 'web-user',
        permissions: ['read', 'write'],
        metadata: {
          generated_by: 'web-interface',
          generation_time: new Date().toISOString(),
          key_type: 'ed25519',
        },
      };

      if (requestBody.expires_at === null) {
        delete requestBody.expires_at;
      }

      for (const key in requestBody.metadata) {
        if (typeof requestBody.metadata[key] !== 'string') {
          requestBody.metadata[key] = String(requestBody.metadata[key]);
        }
      }

      const response = await fetch('/api/security/keys/register', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(requestBody),
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const data = await response.json();
      if (data.success) {
        setIsRegistered(true);
      } else {
        throw new Error(data.error || 'Registration failed');
      }
    } catch (err) {
      setError(`Registration failed: ${err.message}`);
    } finally {
      setIsRegistering(false);
    }
  };

  const copyToClipboard = async (text, field) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopiedField(field);
      setTimeout(() => setCopiedField(null), 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  };

  return (
    <div className="max-w-4xl mx-auto p-6">
      <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
        <div className="flex items-center mb-6">
          <KeyIcon className="h-6 w-6 text-blue-600 mr-2" />
          <h2 className="text-xl font-semibold text-gray-900">Ed25519 Key Generation</h2>
        </div>

        {/* Security Notice */}
        <div className="bg-yellow-50 border border-yellow-200 rounded-md p-4 mb-6">
          <div className="flex">
            <ExclamationTriangleIcon className="h-5 w-5 text-yellow-400 mr-2 flex-shrink-0 mt-0.5" />
            <div className="text-sm text-yellow-700">
              <p className="font-medium">Security Notice:</p>
              <p>Private keys are generated and stored only in your browser's memory. They are never sent to the server. Clear keys when finished.</p>
            </div>
          </div>
        </div>

        {/* Action Buttons */}
        <div className="flex flex-wrap gap-3 mb-6">
          <button
            onClick={generateKeys}
            className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50"
          >
            Generate New Keypair
          </button>

          {publicKeyBase64 && !isRegistered && (
            <button
              onClick={registerPublicKey}
              disabled={isRegistering}
              className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500 disabled:opacity-50"
            >
              {isRegistering ? 'Registering...' : 'Register Public Key'}
            </button>
          )}
          
          {isRegistered && (
            <button
              onClick={() => testSignatureVerification({})}
              disabled={isVerifying}
              className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50"
            >
              <ShieldCheckIcon className="h-5 w-5 mr-2" />
              {isVerifying ? 'Verifying...' : 'Test Signature Verification'}
            </button>
          )}

          {keyPair && (
            <button
              onClick={clearKeys}
              className="inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md shadow-sm text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
            >
              Clear Keys
            </button>
          )}
        </div>

        {/* Error Display */}
        {error && (
          <div className="bg-red-50 border border-red-200 rounded-md p-4 mb-6">
            <div className="flex">
              <ExclamationTriangleIcon className="h-5 w-5 text-red-400 mr-2 flex-shrink-0" />
              <div className="text-sm text-red-700">
                <p className="font-medium">Error:</p>
                <p>{error}</p>
              </div>
            </div>
          </div>
        )}

        {/* Success Message */}
        {isRegistered && (
          <div className="bg-green-50 border border-green-200 rounded-md p-4 mb-6">
            <div className="flex">
              <CheckIcon className="h-5 w-5 text-green-400 mr-2 flex-shrink-0" />
              <div className="text-sm text-green-700">
                <p className="font-medium">Success!</p>
                <p>Public key has been registered with the server.</p>
              </div>
            </div>
          </div>
        )}

        {/* Verification Result */}
        {(verificationResult || verificationError) && (
          <div className={`rounded-md p-4 mb-6 ${verificationResult?.is_valid ? 'bg-green-50 border-green-200' : 'bg-red-50 border-red-200'}`}>
            <div className="flex">
              {verificationResult?.is_valid ? (
                 <ShieldCheckIcon className="h-5 w-5 text-green-400 mr-2 flex-shrink-0" />
              ) : (
                 <ExclamationTriangleIcon className="h-5 w-5 text-red-400 mr-2 flex-shrink-0" />
              )}
              <div className={`text-sm ${verificationResult?.is_valid ? 'text-green-700' : 'text-red-700'}`}>
                <p className="font-medium">Verification Result:</p>
                {verificationResult && (
                  <>
                    <p>Signature Valid: {verificationResult.is_valid ? '✅ Yes' : '❌ No'}</p>
                    {verificationResult.error && <p>Error: {verificationResult.error}</p>}
                    {verificationResult.owner_id && <p>Owner ID: {verificationResult.owner_id}</p>}
                  </>
                )}
                {verificationError && <p>{verificationError}</p>}
              </div>
            </div>
          </div>
        )}

        {/* Key Display */}
          {publicKeyBase64 && (
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Public Key (can be shared safely)
              </label>
              <div className="flex">
                <input
                  type="text"
                  value={publicKeyBase64}
                  readOnly
                  className="flex-1 px-3 py-2 border border-gray-300 rounded-l-md bg-gray-50 text-sm font-mono"
                />
                <button
                  onClick={() => copyToClipboard(publicKeyBase64, 'public')}
                  className="px-3 py-2 border border-l-0 border-gray-300 rounded-r-md bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                  {copiedField === 'public' ? (
                    <CheckIcon className="h-4 w-4 text-green-600" />
                  ) : (
                    <ClipboardIcon className="h-4 w-4 text-gray-500" />
                  )}
                </button>
              </div>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Private Key (keep secret!)
              </label>
              <div className="flex">
                <input
                  type="text"
                  value={"*".repeat(64)}
                  readOnly
                  className="flex-1 px-3 py-2 border border-gray-300 rounded-l-md bg-red-50 text-sm font-mono"
                />
                <button
                  onClick={() => alert("Private key cannot be copied for security reasons.")}
                  className="px-3 py-2 border border-l-0 border-gray-300 rounded-r-md bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                  <ClipboardIcon className="h-4 w-4 text-gray-500" />
                </button>
              </div>
              <p className="text-xs text-red-600 mt-1">
                ⚠️ Never share your private key. It's only shown here for testing purposes.
              </p>
            </div>
          </div>
        )}

        {/* Key Information */}
        {keyPair && (
          <div className="mt-6 bg-gray-50 rounded-md p-4">
            <h3 className="text-sm font-medium text-gray-900 mb-2">Key Information</h3>
            <div className="text-xs text-gray-600 space-y-1">
              <p>• Algorithm: Ed25519</p>
              <p>• Public Key Length: 32 bytes (64 hex characters)</p>
              <p>• Generated: {new Date().toLocaleString()}</p>
              <p>• Registered: {isRegistered ? 'Yes' : 'No'}</p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default KeyGenerationComponent;