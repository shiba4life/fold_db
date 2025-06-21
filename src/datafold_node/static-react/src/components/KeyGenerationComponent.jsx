// React component for Ed25519 key generation

import { useState, useEffect } from 'react';
import { KeyIcon, ClipboardIcon, CheckIcon, ExclamationTriangleIcon } from '@heroicons/react/24/outline';
import * as ed from '@noble/ed25519';

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

const KeyGenerationComponent = () => {
  const [keyPair, setKeyPair] = useState(null);
  const [publicKeyHex, setPublicKeyHex] = useState(null);
  const [privateKeyHex, setPrivateKeyHex] = useState(null);
  const [isGenerating, setIsGenerating] = useState(false);
  const [error, setError] = useState(null);
  const [isRegistered, setIsRegistered] = useState(false);
  const [isRegistering, setIsRegistering] = useState(false);
  const [copiedField, setCopiedField] = useState(null);

  const generateKeys = async () => {
    setIsGenerating(true);
    setError(null);
    setIsRegistered(false);
    
    try {
      const result = await generateKeyPairWithHex();
      setKeyPair(result.keyPair);
      setPublicKeyHex(result.publicKeyHex);
      setPrivateKeyHex(result.privateKeyHex);
    } catch (err) {
      setError(err.message);
    } finally {
      setIsGenerating(false);
    }
  };

  const registerPublicKey = async () => {
    if (!publicKeyHex) return;
    
    setIsRegistering(true);
    setError(null);
    
    try {
      const requestBody = {
        public_key: publicKeyHex,
        owner_id: 'web-user',
        permissions: ['read', 'write'],
        metadata: {
          generated_by: 'web-interface',
          generation_time: new Date().toISOString(),
          key_type: 'ed25519'
        },
        expires_at: null
      };

      const response = await fetch('/api/security/register-key', {
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

  const clearKeys = () => {
    setKeyPair(null);
    setPublicKeyHex(null);
    setPrivateKeyHex(null);
    setError(null);
    setIsRegistered(false);
    setCopiedField(null);
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
            disabled={isGenerating}
            className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50"
          >
            {isGenerating ? 'Generating...' : 'Generate New Keypair'}
          </button>

          {publicKeyHex && !isRegistered && (
            <button
              onClick={registerPublicKey}
              disabled={isRegistering}
              className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500 disabled:opacity-50"
            >
              {isRegistering ? 'Registering...' : 'Register Public Key'}
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

        {/* Key Display */}
        {publicKeyHex && (
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Public Key (can be shared safely)
              </label>
              <div className="flex">
                <input
                  type="text"
                  value={publicKeyHex}
                  readOnly
                  className="flex-1 px-3 py-2 border border-gray-300 rounded-l-md bg-gray-50 text-sm font-mono"
                />
                <button
                  onClick={() => copyToClipboard(publicKeyHex, 'public')}
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
                  value={privateKeyHex}
                  readOnly
                  className="flex-1 px-3 py-2 border border-gray-300 rounded-l-md bg-red-50 text-sm font-mono"
                />
                <button
                  onClick={() => copyToClipboard(privateKeyHex, 'private')}
                  className="px-3 py-2 border border-l-0 border-gray-300 rounded-r-md bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                  {copiedField === 'private' ? (
                    <CheckIcon className="h-4 w-4 text-green-600" />
                  ) : (
                    <ClipboardIcon className="h-4 w-4 text-gray-500" />
                  )}
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
              <p>• Private Key Length: 32 bytes (64 hex characters)</p>
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