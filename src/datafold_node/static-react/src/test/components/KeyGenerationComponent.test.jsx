// Unit tests for KeyGenerationComponent

import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { useState } from 'react';
import KeyGenerationComponent from '../../components/KeyGenerationComponent';
import { registerPublicKey as registerPublicKeyApi } from '../../api/securityClient';
import * as ed from '@noble/ed25519';

// Mock @noble/ed25519 for testing
vi.mock('@noble/ed25519', () => ({
  utils: {
    randomPrivateKey: vi.fn(() => new Uint8Array(32).fill(1)),
  },
  getPublicKeyAsync: vi.fn(() => Promise.resolve(new Uint8Array(32).fill(2))),
}));

vi.mock('../../api/securityClient', () => ({
  registerPublicKey: vi.fn(),
}));

// Test wrapper component that provides state management
function TestWrapper() {
  const [keyPair, setKeyPair] = useState(null);
  const [publicKeyBase64, setPublicKeyBase64] = useState('');
  const [isRegistered, setIsRegistered] = useState(false);
  const [error, setError] = useState(null);
  const [isGenerating, setIsGenerating] = useState(false);

  const generateKeys = async () => {
    setIsGenerating(true);
    setError(null);
    try {
      const privateKey = ed.utils.randomPrivateKey();
      const publicKey = await ed.getPublicKeyAsync(privateKey);
      
      const newKeyPair = { privateKey, publicKey };
      const publicKeyB64 = btoa(String.fromCharCode(...publicKey));
      
      setKeyPair(newKeyPair);
      setPublicKeyBase64(publicKeyB64);
      setIsRegistered(false);
    } catch (err) {
      setError(err.message);
    } finally {
      setIsGenerating(false);
    }
  };

  const clearKeys = () => {
    setKeyPair(null);
    setPublicKeyBase64('');
    setIsRegistered(false);
    setError(null);
  };

  return (
    <KeyGenerationComponent
      keyPair={keyPair}
      publicKeyBase64={publicKeyBase64}
      isRegistered={isRegistered}
      setIsRegistered={setIsRegistered}
      error={error}
      setError={setError}
      generateKeys={generateKeys}
      clearKeys={clearKeys}
      isGenerating={isGenerating}
    />
  );
}

describe('KeyGenerationComponent', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(registerPublicKeyApi).mockResolvedValue({ success: true, public_key_id: 'key123' });
  });

  it('renders the component with initial state', () => {
    render(<TestWrapper />);
    
    expect(screen.getByText('Ed25519 Key Generation')).toBeInTheDocument();
    expect(screen.getByText('Generate New Keypair')).toBeInTheDocument();
    expect(screen.getByText(/Private keys are generated and stored only in your browser's memory/)).toBeInTheDocument();
  });

  it('generates a keypair when button is clicked', async () => {
    render(<TestWrapper />);
    
    const generateButton = screen.getByText('Generate New Keypair');
    fireEvent.click(generateButton);

    // Wait for generation to complete
    await waitFor(() => {
      expect(screen.getByDisplayValue('AgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgI=')).toBeInTheDocument();
    });
  });

  it('shows register button after key generation', async () => {
    render(<TestWrapper />);
    
    const generateButton = screen.getByText('Generate New Keypair');
    fireEvent.click(generateButton);

    await waitFor(() => {
      expect(screen.getByText('Register Public Key')).toBeInTheDocument();
    });
  });

  it('registers public key with the server', async () => {
    render(<TestWrapper />);
    
    // Generate keys first
    const generateButton = screen.getByText('Generate New Keypair');
    fireEvent.click(generateButton);

    await waitFor(() => {
      expect(screen.getByText('Register Public Key')).toBeInTheDocument();
    });

    // Register the key
    const registerButton = screen.getByText('Register Public Key');
    fireEvent.click(registerButton);

    await waitFor(() => {
      expect(registerPublicKeyApi).toHaveBeenCalled();
    });
  });

  it('shows success message after registration', async () => {
    render(<TestWrapper />);
    
    // Generate and register keys
    const generateButton = screen.getByText('Generate New Keypair');
    fireEvent.click(generateButton);

    await waitFor(() => {
      expect(screen.getByText('Register Public Key')).toBeInTheDocument();
    });

    const registerButton = screen.getByText('Register Public Key');
    fireEvent.click(registerButton);

    await waitFor(() => {
      expect(screen.getByText('Success!')).toBeInTheDocument();
      expect(screen.getByText('Public key has been registered with the server.')).toBeInTheDocument();
    });
  });

  it('displays key ID after registration', async () => {
    render(<TestWrapper />);

    const generateButton = screen.getByText('Generate New Keypair');
    fireEvent.click(generateButton);

    await waitFor(() => {
      expect(screen.getByText('Register Public Key')).toBeInTheDocument();
    });

    const registerButton = screen.getByText('Register Public Key');
    fireEvent.click(registerButton);

    await waitFor(() => {
      expect(screen.getByText('Key ID: key123')).toBeInTheDocument();
    });
  });

  it('clears keys when clear button is clicked', async () => {
    render(<TestWrapper />);
    
    // Generate keys first
    const generateButton = screen.getByText('Generate New Keypair');
    fireEvent.click(generateButton);

    await waitFor(() => {
      expect(screen.getByDisplayValue(/AgI/)).toBeInTheDocument();
    });

    // Clear keys
    const clearButton = screen.getByText('Clear Keys');
    fireEvent.click(clearButton);

    // Keys should be gone
    expect(screen.queryByDisplayValue(/AgI/)).not.toBeInTheDocument();
    expect(screen.queryByText('Register Public Key')).not.toBeInTheDocument();
  });

  it('handles registration errors gracefully', async () => {
    // Mock failed API response
    vi.mocked(registerPublicKeyApi).mockResolvedValueOnce({
      success: false,
      error: 'Bad request',
    });

    render(<TestWrapper />);
    
    // Generate keys
    const generateButton = screen.getByText('Generate New Keypair');
    fireEvent.click(generateButton);

    await waitFor(() => {
      expect(screen.getByText('Register Public Key')).toBeInTheDocument();
    });

    // Try to register
    const registerButton = screen.getByText('Register Public Key');
    fireEvent.click(registerButton);

    await waitFor(() => {
      expect(screen.getByText(/Registration failed:/)).toBeInTheDocument();
    });
  });

  it('shows correct key information', async () => {
    render(<TestWrapper />);
    
    const generateButton = screen.getByText('Generate New Keypair');
    fireEvent.click(generateButton);

    await waitFor(() => {
      expect(screen.getByText('Key Information')).toBeInTheDocument();
      expect(screen.getByText('• Algorithm: Ed25519')).toBeInTheDocument();
      expect(screen.getByText('• Private Key Length: 32 bytes (44 base64 characters)')).toBeInTheDocument();
      expect(screen.getByText('• Public Key Length: 32 bytes (44 base64 characters)')).toBeInTheDocument();
    });
  });

  it('provides security warnings for private key', async () => {
    render(<TestWrapper />);
    
    const generateButton = screen.getByText('Generate New Keypair');
    fireEvent.click(generateButton);

    await waitFor(() => {
      expect(screen.getByText('Private Key (keep secret!)')).toBeInTheDocument();
      expect(screen.getByText(/⚠️ Never share your private key/)).toBeInTheDocument();
    });
  });

  it('clears keys on logout event', async () => {
    render(<TestWrapper />);

    const generateButton = screen.getByText('Generate New Keypair');
    fireEvent.click(generateButton);

    await waitFor(() => {
      expect(screen.getByText('Register Public Key')).toBeInTheDocument();
    });

    window.dispatchEvent(new Event('logout'));

    await waitFor(() => {
      expect(screen.queryByText('Register Public Key')).not.toBeInTheDocument();
    });
  });

  it('clears keys on session expiry event', async () => {
    render(<TestWrapper />);

    const generateButton = screen.getByText('Generate New Keypair');
    fireEvent.click(generateButton);

    await waitFor(() => {
      expect(screen.getByText('Register Public Key')).toBeInTheDocument();
    });

    window.dispatchEvent(new Event('session-expired'));

    await waitFor(() => {
      expect(screen.queryByText('Register Public Key')).not.toBeInTheDocument();
    });
  });
});