import React, { useState, useEffect, useCallback } from 'react';

const API_BASE_URL = '/api/security';

const styles = {
  container: {
    fontFamily: 'Arial, sans-serif',
    padding: '20px',
    maxWidth: '600px',
    margin: '0 auto',
    border: '1px solid #ccc',
    borderRadius: '8px',
    boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
  },
  title: {
    textAlign: 'center',
    color: '#333',
  },
  statusContainer: {
    marginBottom: '20px',
    padding: '10px',
    borderRadius: '4px',
  },
  keyPresent: {
    backgroundColor: '#e7f4e7',
    border: '1px solid #c3e6cb',
    color: '#155724',
  },
  noKey: {
    backgroundColor: '#f8d7da',
    border: '1px solid #f5c6cb',
    color: '#721c24',
  },
  keyInfo: {
    wordBreak: 'break-all',
    backgroundColor: '#f0f0f0',
    padding: '10px',
    borderRadius: '4px',
    marginTop: '10px',
  },
  form: {
    display: 'flex',
    flexDirection: 'column',
    gap: '10px',
  },
  textarea: {
    padding: '8px',
    borderRadius: '4px',
    border: '1px solid #ccc',
    minHeight: '100px',
  },
  button: {
    padding: '10px 15px',
    borderRadius: '4px',
    border: 'none',
    cursor: 'pointer',
    color: 'white',
  },
  registerButton: {
    backgroundColor: '#28a745',
  },
  deleteButton: {
    backgroundColor: '#dc3545',
    marginTop: '10px',
  },
  error: {
    color: 'red',
    marginTop: '10px',
  },
  loading: {
    textAlign: 'center',
  },
};

const SecurityKeyManager = () => {
  const [keyInfo, setKeyInfo] = useState(null);
  const [newPublicKey, setNewPublicKey] = useState('');
  const [ownerId, setOwnerId] = useState('');
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState('');

  const fetchKeyStatus = useCallback(async () => {
    setIsLoading(true);
    setError('');
    try {
      const response = await fetch(`${API_BASE_URL}/system-key`);
      if (response.ok) {
        const data = await response.json();
        setKeyInfo(data.key);
      } else {
        setKeyInfo(null);
      }
    } catch (err) {
      setError('Failed to fetch key status.');
      console.error(err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchKeyStatus();
  }, [fetchKeyStatus]);

  const handleRegisterKey = async (e) => {
    e.preventDefault();
    if (!newPublicKey.trim() || !ownerId.trim()) {
      setError('Public key and Owner ID are required.');
      return;
    }
    setError('');
    try {
      const response = await fetch(`${API_BASE_URL}/system-key`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          public_key: newPublicKey,
          owner_id: ownerId,
          permissions: ['read', 'write', 'admin'], // Defaulting to all permissions
        }),
      });
      if (response.ok) {
        setNewPublicKey('');
        setOwnerId('');
        fetchKeyStatus();
      } else {
        const errData = await response.json();
        setError(errData.error || 'Failed to register key.');
      }
    } catch (err) {
      setError('An error occurred while registering the key.');
      console.error(err);
    }
  };

  const handleDeleteKey = async () => {
    if (window.confirm('Are you sure you want to delete the system key? This action cannot be undone.')) {
      setError('');
      try {
        const response = await fetch(`${API_BASE_URL}/system-key`, {
          method: 'DELETE',
        });
        if (response.ok) {
          fetchKeyStatus();
        } else {
          const errData = await response.json();
          setError(errData.error || 'Failed to delete key.');
        }
      } catch (err) {
        setError('An error occurred while deleting the key.');
        console.error(err);
      }
    }
  };

  if (isLoading) {
    return <div style={styles.loading}>Loading...</div>;
  }

  return (
    <div style={styles.container}>
      <h2 style={styles.title}>System-Wide Security Key Management</h2>
      
      <div style={{ ...styles.statusContainer, ...(keyInfo ? styles.keyPresent : styles.noKey) }}>
        {keyInfo ? (
          <div>
            <strong>Status:</strong> A system-wide public key is registered.
            <div style={styles.keyInfo}>
              <p><strong>Owner ID:</strong> {keyInfo.owner_id}</p>
              <p><strong>Public Key:</strong> {keyInfo.public_key}</p>
              <p><strong>Permissions:</strong> {keyInfo.permissions.join(', ')}</p>
            </div>
            <button onClick={handleDeleteKey} style={{...styles.button, ...styles.deleteButton}}>
              Delete System Key
            </button>
          </div>
        ) : (
          <strong>Status:</strong> No system-wide public key is registered.
        )}
      </div>

      <form onSubmit={handleRegisterKey} style={styles.form}>
        <h3>{keyInfo ? 'Replace' : 'Register'} System Key</h3>
        <input
          type="text"
          placeholder="Owner ID (e.g., admin-user)"
          value={ownerId}
          onChange={(e) => setOwnerId(e.target.value)}
          style={styles.textarea}
        />
        <textarea
          placeholder="Enter new Ed25519 Public Key (Base64)"
          value={newPublicKey}
          onChange={(e) => setNewPublicKey(e.target.value)}
          style={styles.textarea}
        />
        <button type="submit" style={{...styles.button, ...styles.registerButton}}>
          {keyInfo ? 'Replace Key' : 'Register Key'}
        </button>
      </form>

      {error && <p style={styles.error}>{error}</p>}
    </div>
  );
};

export default SecurityKeyManager; 