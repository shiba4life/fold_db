// Key Management Tab wrapper component

import React, { useState } from 'react';
import DataStorageForm from '../DataStorageForm';

function KeyManagementTab({ onResult, keyGenerationResult }) {
    const [isRegistering, setIsRegistering] = useState(false);
    const [registrationStatus, setRegistrationStatus] = useState(null); // State for feedback

    // Destructure the state and functions from the prop
    const { result, generateKeyPair, clearKeys, registerPublicKey } = keyGenerationResult;
    const { keyPair, publicKeyHex, error, isGenerating } = result;

    const handleRegister = async () => {
        if (!publicKeyHex) return;
        setIsRegistering(true);
        setRegistrationStatus(null); // Clear previous status
        try {
            const success = await registerPublicKey(publicKeyHex);
            const status = {
                success,
                message: success ? 'Public key registered successfully!' : 'Failed to register public key.',
            };
            setRegistrationStatus(status);
            onResult(status); // Also update the main result panel
        } catch (e) {
            const status = { success: false, message: e.message };
            setRegistrationStatus(status);
            onResult(status);
        }
        setIsRegistering(false);
    };

    return (
        <div className="p-4 bg-white rounded-lg shadow">
            <h2 className="text-xl font-semibold mb-4">Key Management</h2>
            <div className="flex space-x-2 mb-4">
                <button
                    onClick={generateKeyPair}
                    disabled={isGenerating}
                    className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:bg-gray-400"
                >
                    {isGenerating ? 'Generating...' : 'Generate New Keypair'}
                </button>
                <button
                    onClick={clearKeys}
                    disabled={!publicKeyHex}
                    className="px-4 py-2 bg-gray-500 text-white rounded hover:bg-gray-600 disabled:bg-gray-400"
                >
                    Clear Keys
                </button>
                <button
                    onClick={handleRegister}
                    disabled={!publicKeyHex || isRegistering}
                    className="px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600 disabled:bg-gray-400"
                >
                    {isRegistering ? 'Registering...' : 'Register Public Key'}
                </button>
            </div>

            {registrationStatus && (
                <div
                    className={`p-4 mb-4 text-sm rounded-lg ${
                        registrationStatus.success
                            ? 'text-green-700 bg-green-100'
                            : 'text-red-700 bg-red-100'
                    }`}
                    role="alert"
                >
                    <span className="font-medium">
                        {registrationStatus.success ? 'Success!' : 'Error:'}
                    </span>{' '}
                    {registrationStatus.message}
                </div>
            )}

            {error && (
                <div className="p-4 mb-4 text-sm text-red-700 bg-red-100 rounded-lg" role="alert">
                    <span className="font-medium">Error:</span> {error}
                </div>
            )}

            {publicKeyHex && (
                <div className="key-display">
                    <h3 className="text-lg font-medium">Generated Public Key (Base64)</h3>
                    <textarea
                        readOnly
                        value={publicKeyHex}
                        className="w-full h-24 p-2 mt-2 border border-gray-300 rounded-md bg-gray-50"
                    />
                </div>
            )}

            {keyPair && publicKeyHex && (
                <div className="border-t border-gray-200 mt-6 pt-6">
                    <DataStorageForm keyPair={keyPair} publicKeyHex={publicKeyHex} />
                </div>
            )}
        </div>
    );
}

export default KeyManagementTab;