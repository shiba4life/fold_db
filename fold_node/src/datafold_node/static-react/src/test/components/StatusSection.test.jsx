import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import StatusSection from '../../components/StatusSection'

// Mock fetch globally
global.fetch = vi.fn()

// Mock window.confirm and window.alert
global.confirm = vi.fn()
global.alert = vi.fn()

describe('StatusSection Component', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })
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

  it('displays restart buttons', () => {
    render(<StatusSection />)
    
    const softRestartButton = screen.getByText('Soft Restart')
    const fullRestartButton = screen.getByText('Full Restart')
    
    expect(softRestartButton).toBeInTheDocument()
    expect(fullRestartButton).toBeInTheDocument()
    
    expect(softRestartButton).toHaveClass('bg-blue-50', 'text-blue-700', 'hover:bg-blue-100')
    expect(fullRestartButton).toHaveClass('bg-red-50', 'text-red-700', 'hover:bg-red-100')
  })

  it('shows confirmation dialog when restart buttons are clicked', async () => {
    global.confirm.mockReturnValue(false) // User cancels
    render(<StatusSection />)
    
    const softRestartButton = screen.getByText('Soft Restart')
    const fullRestartButton = screen.getByText('Full Restart')
    
    fireEvent.click(softRestartButton)
    expect(global.confirm).toHaveBeenCalledWith(
      'Are you sure you want to soft restart the fold_node? This will reinitialize the database while preserving network connections.'
    )
    
    fireEvent.click(fullRestartButton)
    expect(global.confirm).toHaveBeenCalledWith(
      'Are you sure you want to full restart the fold_node? This will stop all services and reinitialize everything.'
    )
  })

  it('calls soft restart API when soft restart button is clicked', async () => {
    global.confirm.mockReturnValue(true) // User confirms
    global.fetch.mockResolvedValue({
      ok: true,
      json: () => Promise.resolve({ success: true })
    })
    
    render(<StatusSection />)
    
    const softRestartButton = screen.getByText('Soft Restart')
    fireEvent.click(softRestartButton)
    
    await waitFor(() => {
      expect(global.fetch).toHaveBeenCalledWith('/api/system/soft-restart', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
      })
    })
  })

  it('calls full restart API when full restart button is clicked', async () => {
    global.confirm.mockReturnValue(true) // User confirms
    global.fetch.mockResolvedValue({
      ok: true,
      json: () => Promise.resolve({ success: true })
    })
    
    render(<StatusSection />)
    
    const fullRestartButton = screen.getByText('Full Restart')
    fireEvent.click(fullRestartButton)
    
    await waitFor(() => {
      expect(global.fetch).toHaveBeenCalledWith('/api/system/restart', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
      })
    })
  })

  it('shows loading states during restart operations', async () => {
    global.confirm.mockReturnValue(true)
    global.fetch.mockImplementation(() => new Promise(() => {})) // Never resolves
    
    render(<StatusSection />)
    
    const softRestartButton = screen.getByText('Soft Restart')
    fireEvent.click(softRestartButton)
    
    await waitFor(() => {
      expect(screen.getByText('Soft Restarting...')).toBeInTheDocument()
    })
  })
})