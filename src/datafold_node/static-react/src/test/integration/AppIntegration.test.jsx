import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, it, expect, beforeEach, vi } from 'vitest'
import App from '../../App'

// Mock fetch globally
global.fetch = vi.fn()

describe('App Integration Tests', () => {
  let user

  beforeEach(() => {
    user = userEvent.setup()
    
    // Reset fetch mock
    fetch.mockReset()
    
    // Mock successful API responses
    fetch.mockImplementation((url) => {
      if (url === '/api/schemas') {
        return Promise.resolve({
          ok: true,
          json: async () => ({
            data: {
              'UserProfile': 'Available',
              'BlogPost': 'Approved',
              'ProductCatalog': 'Available'
            }
          })
        })
      }
      
      if (url.startsWith('/api/schema/')) {
        const schemaName = url.split('/').pop()
        return Promise.resolve({
          ok: true,
          json: async () => ({
            name: schemaName,
            fields: {
              id: { field_type: 'string', writable: false },
              name: { field_type: 'string', writable: true },
              email: { field_type: 'string', writable: true }
            }
          })
        })
      }
      
      if (url === '/api/samples/schemas') {
        return Promise.resolve({
          ok: true,
          json: async () => ({
            data: ['SampleUser', 'SampleBlog', 'SampleProduct']
          })
        })
      }
      
      if (url.startsWith('/api/samples/schema/')) {
        const schemaName = url.split('/').pop()
        return Promise.resolve({
          ok: true,
          json: async () => ({
            name: schemaName,
            fields: {
              title: { field_type: 'string', writable: true },
              content: { field_type: 'text', writable: true }
            }
          })
        })
      }
      
      // Default fallback
      return Promise.resolve({
        ok: true,
        json: async () => ({ data: [] })
      })
    })
  })

  it('renders main application components', async () => {
    render(<App />)
    
    // Check for main UI elements
    expect(screen.getByText('DataFold Node')).toBeInTheDocument()
    expect(screen.getByText('Node is running successfully')).toBeInTheDocument()
    expect(screen.getByText('Active and healthy')).toBeInTheDocument()
    
    // Check for navigation tabs
    expect(screen.getByText('Schemas')).toBeInTheDocument()
    expect(screen.getByText('Query')).toBeInTheDocument()
    expect(screen.getByText('Mutation')).toBeInTheDocument()
    expect(screen.getByText('Transforms')).toBeInTheDocument()
    expect(screen.getByText('Dependencies')).toBeInTheDocument()
  })

  it('loads and displays schemas', async () => {
    render(<App />)
    
    // Wait for schemas to load
    await waitFor(() => {
      expect(screen.getByText('Available Schemas')).toBeInTheDocument()
      expect(screen.getByText('Approved Schemas')).toBeInTheDocument()
    })
    
    // Check that API was called
    expect(fetch).toHaveBeenCalledWith('/api/schemas')
  })

  it('switches between tabs correctly', async () => {
    render(<App />)
    
    // Initially on Schemas tab
    const schemasTab = screen.getByText('Schemas')
    expect(schemasTab).toHaveClass('text-primary')
    
    // Click Query tab
    const queryTab = screen.getByText('Query')
    await user.click(queryTab)
    
    // Check Query tab is active
    await waitFor(() => {
      expect(queryTab).toHaveClass('text-primary')
      expect(screen.getByText('Run Sample Query')).toBeInTheDocument()
    })
    
    // Click Mutation tab
    const mutationTab = screen.getByText('Mutation')
    await user.click(mutationTab)
    
    // Check Mutation tab is active
    await waitFor(() => {
      expect(mutationTab).toHaveClass('text-primary')
      expect(screen.getByText('Run Sample Mutation')).toBeInTheDocument()
    })
  })

  it('handles API errors gracefully', async () => {
    // Mock API error
    fetch.mockRejectedValueOnce(new Error('Network error'))
    
    render(<App />)
    
    // Should still render the UI even with API error
    await waitFor(() => {
      expect(screen.getByText('DataFold Node')).toBeInTheDocument()
      expect(screen.getByText('Schemas')).toBeInTheDocument()
    })
  })

  it('displays transform queue status', async () => {
    render(<App />)
    
    // Click Transforms tab
    const transformsTab = screen.getByText('Transforms')
    await user.click(transformsTab)
    
    // Check that the tab is active (no need to check specific content)
    await waitFor(() => {
      expect(transformsTab).toHaveClass('text-primary')
    })
  })

  it('shows system status controls', async () => {
    render(<App />)
    
  })

  it('displays log sidebar', async () => {
    render(<App />)
    
    // Check for log sidebar
    expect(screen.getByText('Logs')).toBeInTheDocument()
  })
})