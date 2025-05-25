import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { describe, it, expect, beforeEach, vi } from 'vitest'
import SchemasTab from '../../../components/tabs/SchemasTab'

describe('SchemasTab Component', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    fetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({ data: [{ name: 'User' }] })
    }).mockResolvedValueOnce({
      ok: true,
      json: async () => ({ data: ['User', 'Blog'] })
    })
  })

  it('fetches loaded and available schemas on mount', async () => {
    render(<SchemasTab />)

    await waitFor(() => {
      expect(fetch).toHaveBeenCalledWith('/api/schemas')
      expect(fetch).toHaveBeenCalledWith('/api/schemas/available')
    })
  })

  it('unloads schema on button click', async () => {
    fetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({ data: [{ name: 'User' }] })
    }).mockResolvedValueOnce({
      ok: true,
      json: async () => ({ data: [] })
    }).mockResolvedValueOnce({ ok: true })

    render(<SchemasTab />)

    await waitFor(() => {
      expect(screen.getByText('User')).toBeInTheDocument()
    })

    const btn = screen.getByText('Unload')
    fireEvent.click(btn)

    await waitFor(() => {
      expect(fetch).toHaveBeenCalledWith('/api/schema/User', { method: 'DELETE' })
    })
  })
})
