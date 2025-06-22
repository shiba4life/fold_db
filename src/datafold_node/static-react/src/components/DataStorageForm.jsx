import React, { useState } from 'react';
import { PaperAirplaneIcon, ExclamationTriangleIcon, ShieldCheckIcon } from '@heroicons/react/24/outline';
import { useSigning } from '../hooks/useSigning';

const DataStorageForm = ({ keyPair, publicKeyBase64 }) => {
  const [value1, setValue1] = useState('sample-value-1');
  const [value2, setValue2] = useState('sample-value-2');
  const [mutationResult, setMutationResult] = useState(null);
  const [mutationError, setMutationError] = useState(null);
  const [isLoading, setIsLoading] = useState(false);
  const { signPayload } = useSigning();

  const handleSubmit = async (e) => {
    e.preventDefault();
    setMutationResult(null);
    setMutationError(null);
    setIsLoading(true);

    if (!keyPair || !publicKeyBase64) {
      setMutationError("Keypair not available. Please generate and register a key first.");
      setIsLoading(false);
      return;
    }

    const mutationPayload = {
      type: 'mutation',
      schema: 'TransformBase',
      mutation_type: 'create',
      data: {
        value1: value1,
        value2: value2,
      },
    };
    
    try {
        const signedMessage = await signPayload(
          mutationPayload,
          publicKeyBase64,
          keyPair.privateKey
        );

        const response = await fetch('/api/mutation', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(signedMessage),
        });

        const data = await response.json();

        if (!response.ok) {
            throw new Error(data.error || `HTTP error! status: ${response.status}`);
        }

        setMutationResult(data);

    } catch (err) {
        setMutationError(err.message);
    } finally {
        setIsLoading(false);
    }
  };

  return (
    <div className="max-w-4xl mx-auto p-6">
      <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
        <h2 className="text-xl font-semibold text-gray-900 mb-4">Secure Data Mutation: Create Transform</h2>
        <p className="text-sm text-gray-600 mb-6">
          This form demonstrates sending a signed data mutation to the backend. The transform creation will be packaged into a mutation, signed on the client-side with your private key, and sent to the server for verification and processing.
        </p>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label htmlFor="value1" className="block text-sm font-medium text-gray-700">Value 1</label>
            <input
              type="text"
              id="value1"
              value={value1}
              onChange={(e) => setValue1(e.target.value)}
              className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
            />
          </div>
          <div>
            <label htmlFor="value2" className="block text-sm font-medium text-gray-700">Value 2</label>
            <input
              type="text"
              id="value2"
              value={value2}
              onChange={(e) => setValue2(e.target.value)}
              className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
            />
          </div>
          <div>
            <button
              type="submit"
              disabled={isLoading || !keyPair}
              className="w-full inline-flex items-center justify-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50"
            >
              <PaperAirplaneIcon className="h-5 w-5 mr-2" />
              {isLoading ? 'Sending...' : 'Sign and Submit Transform Data'}
            </button>
          </div>
        </form>

        {mutationResult && (
           <div className="mt-6 bg-green-50 border border-green-200 rounded-md p-4">
             <div className="flex">
               <ShieldCheckIcon className="h-5 w-5 text-green-400 mr-2 flex-shrink-0" />
               <div className="text-sm text-green-700">
                 <p className="font-medium">Mutation Success!</p>
                 <pre className="text-xs whitespace-pre-wrap">{JSON.stringify(mutationResult, null, 2)}</pre>
               </div>
             </div>
           </div>
        )}
        
        {mutationError && (
            <div className="mt-6 bg-red-50 border border-red-200 rounded-md p-4">
                <div className="flex">
                    <ExclamationTriangleIcon className="h-5 w-5 text-red-400 mr-2 flex-shrink-0" />
                    <div className="text-sm text-red-700">
                        <p className="font-medium">Error</p>
                        <p>{mutationError}</p>
                    </div>
                </div>
            </div>
        )}

      </div>
    </div>
  );
};

export default DataStorageForm; 