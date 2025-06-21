import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, it, expect, beforeEach, vi } from 'vitest'
import React from 'react'
import { useKeyGeneration } from '../../hooks/useKeyGeneration'
import KeyManagementTab from '../../components/tabs/KeyManagementTab'

// Mock deterministic Ed25519 functions
vi.mock('@noble/ed25519', () => ({
  utils: { randomPrivateKey: vi.fn(() => new Uint8Array(32).fill(1)) },
  getPublicKeyAsync: vi.fn(() => Promise.resolve(new Uint8Array(32).fill(2))),
  signAsync: vi.fn(() => Promise.resolve(new Uint8Array(64).fill(3)))
}))

function Wrapper() {
  const keyGen = useKeyGeneration()
  return <KeyManagementTab onResult={() => {}} keyGenerationResult={keyGen} />
}

describe('Key lifecycle workflow', () => {
  let user
  beforeEach(() => {
    user = userEvent.setup()
    global.fetch = vi.fn((url) => {
      if (url === '/api/security/keys/register') {
        return Promise.resolve({ ok: true, json: () => Promise.resolve({ success: true }) })
      }
      if (url === '/api/data/mutate') {
        return Promise.resolve({ ok: true, json: () => Promise.resolve({ ok: true }) })
      }
      return Promise.resolve({ ok: true, json: () => Promise.resolve({}) })
    })
  })

  it('generates, registers, signs and clears keys', async () => {
    render(<Wrapper />)
    await user.click(screen.getByText('Generate New Keypair'))

    await waitFor(() => {
      expect(screen.getByText('Register Public Key')).toBeInTheDocument()
    })

    await user.click(screen.getByText('Register Public Key'))
    await waitFor(() => {
      expect(fetch).toHaveBeenCalledWith('/api/security/keys/register', expect.any(Object))
      expect(screen.getByText(/registered successfully/i)).toBeInTheDocument()
    })

    await user.type(screen.getByLabelText('Post ID'), 'post-abc')
    await user.type(screen.getByLabelText('Your User ID (Liker)'), 'user-xyz')
    await user.click(screen.getByRole('button', { name: /Sign and Send Like/i }))

    await waitFor(() => {
      expect(fetch).toHaveBeenCalledWith('/api/data/mutate', expect.any(Object))
    })

    await user.click(screen.getByText('Clear Keys'))
    await waitFor(() => {
      expect(screen.queryByRole('button', { name: /Sign and Send Like/i })).not.toBeInTheDocument()
    })
  })
})
