// Unit tests for KeyGenerationComponent

import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import KeyGenerationComponent from '../../components/KeyGenerationComponent';

// Mock @noble/ed25519 for testing
vi.mock('@noble/ed25519', () => ({
  utils: {
    randomPrivateKey: vi.fn(() => new Uint8Array(32).fill(1)),
  },
  getPublicKeyAsync: vi.fn(() => Promise.resolve(new Uint8Array(32).fill(2))),
}));

// Mock fetch for API calls
global.fetch = vi.fn();

describe('KeyGenerationComponent', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Setup default fetch response
    global.fetch.mockResolvedValue({
      ok: true,
      json: () => Promise.resolve({ success: true, public_key_id: 'key123' }),
    });
  });

  it('renders the component with initial state', () => {
    render(<KeyGenerationComponent />);
    
    expect(screen.getByText('Ed25519 Key Generation')).toBeInTheDocument();
    expect(screen.getByText('Generate New Keypair')).toBeInTheDocument();
    expect(screen.getByText(/Private keys are generated and stored only in your browser's memory/)).toBeInTheDocument();
  });

  it('generates a keypair when button is clicked', async () => {
    render(<KeyGenerationComponent />);
    
    const generateButton = screen.getByText('Generate New Keypair');
    fireEvent.click(generateButton);

    // Should show generating state
    expect(screen.getByText('Generating...')).toBeInTheDocument();

    // Wait for generation to complete
    await waitFor(() => {
      expect(screen.getByDisplayValue('0202020202020202020202020202020202020202020202020202020202020202')).toBeInTheDocument();
    });

    // Should show public and private keys
    expect(screen.getByDisplayValue('0202020202020202020202020202020202020202020202020202020202020202')).toBeInTheDocument();
    expect(screen.getByDisplayValue('0101010101010101010101010101010101010101010101010101010101010101')).toBeInTheDocument();
  });

  it('shows register button after key generation', async () => {
    render(<KeyGenerationComponent />);
    
    const generateButton = screen.getByText('Generate New Keypair');
    fireEvent.click(generateButton);

    await waitFor(() => {
      expect(screen.getByText('Register Public Key')).toBeInTheDocument();
    });
  });

  it('registers public key with the server', async () => {
    render(<KeyGenerationComponent />);
    
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
      expect(global.fetch).toHaveBeenCalledWith('/api/security/keys/register', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: expect.stringContaining('0202020202020202020202020202020202020202020202020202020202020202'),
      });
    });
  });

  it('shows success message after registration', async () => {
    render(<KeyGenerationComponent />);
    
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
    render(<KeyGenerationComponent />);

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
    render(<KeyGenerationComponent />);
    
    // Generate keys first
    const generateButton = screen.getByText('Generate New Keypair');
    fireEvent.click(generateButton);

    await waitFor(() => {
      expect(screen.getByDisplayValue(/020/)).toBeInTheDocument();
    });

    // Clear keys
    const clearButton = screen.getByText('Clear Keys');
    fireEvent.click(clearButton);

    // Keys should be gone
    expect(screen.queryByDisplayValue(/020/)).not.toBeInTheDocument();
    expect(screen.queryByText('Register Public Key')).not.toBeInTheDocument();
  });

  it('handles registration errors gracefully', async () => {
    // Mock failed API response
    global.fetch.mockResolvedValueOnce({
      ok: false,
      status: 400,
    });

    render(<KeyGenerationComponent />);
    
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
    render(<KeyGenerationComponent />);
    
    const generateButton = screen.getByText('Generate New Keypair');
    fireEvent.click(generateButton);

    await waitFor(() => {
      expect(screen.getByText('Key Information')).toBeInTheDocument();
      expect(screen.getByText('• Algorithm: Ed25519')).toBeInTheDocument();
      expect(screen.getByText('• Private Key Length: 32 bytes (64 hex characters)')).toBeInTheDocument();
      expect(screen.getByText('• Public Key Length: 32 bytes (64 hex characters)')).toBeInTheDocument();
    });
  });

  it('provides security warnings for private key', async () => {
    render(<KeyGenerationComponent />);
    
    const generateButton = screen.getByText('Generate New Keypair');
    fireEvent.click(generateButton);

    await waitFor(() => {
      expect(screen.getByText('Private Key (keep secret!)')).toBeInTheDocument();
      expect(screen.getByText(/Never share your private key/)).toBeInTheDocument();
    });
  });
});