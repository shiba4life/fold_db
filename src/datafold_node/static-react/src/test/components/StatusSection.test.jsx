import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { describe, it, expect, beforeEach, vi } from 'vitest'
import StatusSection from '../../components/StatusSection'

describe('StatusSection Component', () => {
  it('renders status message correctly', () => {
    render(<StatusSection />)
    
    expect(screen.getByText('Node is running successfully')).toBeInTheDocument()
    expect(screen.getByText('Active and healthy')).toBeInTheDocument()
  })

  it('has correct container styling', () => {
    render(<StatusSection />)
    
    const container = screen.getByText('Node is running successfully').closest('div').parentElement.parentElement
    expect(container).toHaveClass('bg-white', 'rounded-lg', 'shadow-sm', 'p-4', 'mb-6')
  })

  it('displays check circle icon', () => {
    render(<StatusSection />)
    
    // The CheckCircleIcon should be rendered as an SVG
    const icon = screen.getByText('Node is running successfully').parentElement.querySelector('svg')
    expect(icon).toBeInTheDocument()
    expect(icon).toHaveClass('icon', 'icon-md', 'text-green-500')
  })

  it('has proper layout structure', () => {
    render(<StatusSection />)
    
    const mainContainer = screen.getByText('Node is running successfully').parentElement
    expect(mainContainer).toHaveClass('flex', 'items-center', 'gap-3')
    
    const statusContainer = screen.getByText('Active and healthy').parentElement
    expect(statusContainer).toHaveClass('mt-2', 'flex', 'items-center', 'gap-2')
  })

  it('displays animated pulse indicator', () => {
    render(<StatusSection />)
    
    const pulseIndicator = screen.getByText('Active and healthy').parentElement.querySelector('.animate-pulse')
    expect(pulseIndicator).toBeInTheDocument()
    expect(pulseIndicator).toHaveClass('w-2', 'h-2', 'rounded-full', 'bg-green-500', 'animate-pulse')
  })

  it('has correct text styling', () => {
    render(<StatusSection />)
    
    const mainText = screen.getByText('Node is running successfully')
    expect(mainText).toHaveClass('text-gray-700', 'font-medium')
    
    const subText = screen.getByText('Active and healthy')
    expect(subText).toHaveClass('text-sm', 'text-gray-500')
  })

  it('renders all visual elements', () => {
    render(<StatusSection />)
    
    // Check that all key elements are present
    expect(screen.getByText('Node is running successfully')).toBeInTheDocument()
    expect(screen.getByText('Active and healthy')).toBeInTheDocument()
    
    // Check for icon
    const icon = screen.getByText('Node is running successfully').parentElement.querySelector('svg')
    expect(icon).toBeInTheDocument()
    
    // Check for pulse indicator
    const pulseIndicator = screen.getByText('Active and healthy').parentElement.querySelector('.animate-pulse')
    expect(pulseIndicator).toBeInTheDocument()
  })

  describe('Database Reset Functionality', () => {
    beforeEach(() => {
      // Reset all mocks before each test
      vi.clearAllMocks()
      
      // Mock fetch globally
      global.fetch = vi.fn()
    })

    it('renders reset database button', () => {
      render(<StatusSection />)
      
      const resetButton = screen.getByRole('button', { name: /reset database/i })
      expect(resetButton).toBeInTheDocument()
      expect(resetButton).toHaveClass('text-red-600', 'border-red-200')
    })

    it('shows confirmation dialog when reset button is clicked', () => {
      render(<StatusSection />)
      
      const resetButton = screen.getByRole('button', { name: /reset database/i })
      fireEvent.click(resetButton)
      
      expect(screen.getByRole('heading', { name: /reset database/i })).toBeInTheDocument()
      expect(screen.getByText(/This will permanently delete all data/)).toBeInTheDocument()
      expect(screen.getByText(/All schemas will be removed/)).toBeInTheDocument()
      expect(screen.getByText(/This action cannot be undone/)).toBeInTheDocument()
    })

    it('closes confirmation dialog when cancel is clicked', () => {
      render(<StatusSection />)
      
      const resetButton = screen.getByRole('button', { name: /reset database/i })
      fireEvent.click(resetButton)
      
      const cancelButton = screen.getByRole('button', { name: /cancel/i })
      fireEvent.click(cancelButton)
      
      expect(screen.queryByRole('heading', { name: /reset database/i })).not.toBeInTheDocument()
    })

    it('calls reset API when confirmed', async () => {
      global.fetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: true, message: 'Database reset successfully' })
      })

      render(<StatusSection />)
      
      const resetButton = screen.getByRole('button', { name: /reset database/i })
      fireEvent.click(resetButton)
      
      const confirmButton = screen.getAllByRole('button', { name: /reset database/i })[1]
      fireEvent.click(confirmButton)
      
      await waitFor(() => {
        expect(global.fetch).toHaveBeenCalledWith('/api/system/reset-database', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({ confirm: true }),
        })
      })
    })

    it('shows success message when reset succeeds', async () => {
      global.fetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: true, message: 'Database reset successfully' })
      })

      render(<StatusSection />)
      
      const resetButton = screen.getByRole('button', { name: /reset database/i })
      fireEvent.click(resetButton)
      
      const confirmButton = screen.getAllByRole('button', { name: /reset database/i })[1] // Get the modal button
      fireEvent.click(confirmButton)
      
      await waitFor(() => {
        expect(screen.getByText('Database reset successfully')).toBeInTheDocument()
      })
    })

    it('shows error message when reset fails', async () => {
      global.fetch.mockResolvedValueOnce({
        ok: false,
        json: async () => ({ success: false, message: 'Reset failed' })
      })

      render(<StatusSection />)
      
      const resetButton = screen.getByRole('button', { name: /reset database/i })
      fireEvent.click(resetButton)
      
      const confirmButton = screen.getAllByRole('button', { name: /reset database/i })[1] // Get the modal button
      fireEvent.click(confirmButton)
      
      await waitFor(() => {
        expect(screen.getByText('Reset failed')).toBeInTheDocument()
      })
    })

    it('handles network errors gracefully', async () => {
      global.fetch.mockRejectedValueOnce(new Error('Network error'))

      render(<StatusSection />)
      
      const resetButton = screen.getByRole('button', { name: /reset database/i })
      fireEvent.click(resetButton)
      
      const confirmButton = screen.getAllByRole('button', { name: /reset database/i })[1] // Get the modal button
      fireEvent.click(confirmButton)
      
      await waitFor(() => {
        expect(screen.getByText(/Network error/)).toBeInTheDocument()
      })
    })

    it('disables reset button while resetting', async () => {
      global.fetch.mockImplementationOnce(() => new Promise(resolve => setTimeout(resolve, 1000)))

      render(<StatusSection />)
      
      const resetButton = screen.getByRole('button', { name: /reset database/i })
      fireEvent.click(resetButton)
      
      const confirmButton = screen.getAllByRole('button', { name: /reset database/i })[1] // Get the modal button
      fireEvent.click(confirmButton)
      
      // Button should show "Resetting..." and be disabled
      await waitFor(() => {
        expect(screen.getByText('Resetting...')).toBeInTheDocument()
      })
      
      const disabledButton = screen.getByRole('button', { name: /resetting/i })
      expect(disabledButton).toBeDisabled()
    })

    it('shows proper button styling for destructive action', () => {
      render(<StatusSection />)
      
      const resetButton = screen.getByRole('button', { name: /reset database/i })
      fireEvent.click(resetButton)
      
      const confirmButton = screen.getAllByRole('button', { name: /reset database/i })[1] // Get the modal button
      expect(confirmButton).toHaveClass('bg-red-600', 'text-white', 'hover:bg-red-700')
    })

    it('includes trash icon in reset button', () => {
      render(<StatusSection />)
      
      const resetButton = screen.getByRole('button', { name: /reset database/i })
      const icon = resetButton.querySelector('svg')
      expect(icon).toBeInTheDocument()
      expect(icon).toHaveClass('w-4', 'h-4')
    })

    it('confirms dialog accessibility features', () => {
      render(<StatusSection />)
      
      const resetButton = screen.getByRole('button', { name: /reset database/i })
      fireEvent.click(resetButton)
      
      // Check for proper heading
      expect(screen.getByRole('heading', { level: 3 })).toHaveTextContent('Reset Database')
      
      // Check for proper button roles
      expect(screen.getByRole('button', { name: /cancel/i })).toBeInTheDocument()
      expect(screen.getAllByRole('button', { name: /reset database/i })[1]).toBeInTheDocument() // Get the modal button
    })
  })
})