import React from 'react';

const PrivateKeyManager = ({ privateKey, selectPrivateKeyFile }) => {
  return (
    <div className="private-key-manager">
      <h1 className="mb-4">Private Key Manager</h1>
      
      <div className="card mb-4">
        <div className="card-header">
          <i className="fas fa-key me-2"></i>
          Private Key
        </div>
        <div className="card-body">
          {privateKey ? (
            <div className="private-key-section">
              <h5>Current Private Key</h5>
              <p><strong>File:</strong> {privateKey.path}</p>
              <div className="private-key-display">
                {privateKey.content}
              </div>
              
              <div className="mt-3">
                <button 
                  className="btn btn-primary me-2" 
                  onClick={selectPrivateKeyFile}
                >
                  <i className="fas fa-exchange-alt me-2"></i>
                  Change Private Key
                </button>
              </div>
            </div>
          ) : (
            <div className="private-key-section">
              <div className="alert alert-info">
                <i className="fas fa-info-circle me-2"></i>
                No private key loaded. Select a private key file to continue.
              </div>
              
              <button 
                className="btn btn-primary" 
                onClick={selectPrivateKeyFile}
              >
                <i className="fas fa-file-import me-2"></i>
                Select Private Key File
              </button>
            </div>
          )}
        </div>
      </div>
      
      <div className="card mb-4">
        <div className="card-header">
          <i className="fas fa-info-circle me-2"></i>
          About Private Keys
        </div>
        <div className="card-body">
          <h5>Why do I need a private key?</h5>
          <p>
            The private key is used to authenticate your FoldClient with the DataFold node.
            It ensures that only authorized clients can access the node API.
          </p>
          
          <h5>How to generate a private key</h5>
          <p>
            You can generate a private key using OpenSSL with the following command:
          </p>
          <div className="bg-light p-3 rounded mb-3">
            <code>openssl genpkey -algorithm ed25519 -out fold_client_private.pem</code>
          </div>
          
          <h5>Security considerations</h5>
          <ul>
            <li>Keep your private key secure and do not share it with others.</li>
            <li>Store your private key in a secure location.</li>
            <li>Consider using a password-protected private key for additional security.</li>
          </ul>
        </div>
      </div>
    </div>
  );
};

export default PrivateKeyManager;
