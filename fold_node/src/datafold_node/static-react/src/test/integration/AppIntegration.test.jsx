import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, it, expect, beforeEach, vi } from 'vitest'
import App from '../../App'

describe('App Integration Tests', () => {
  let user

  beforeEach(() => {
    user = userEvent.setup()
    
    // Mock successful API responses
    fetch.mockImplementation((url) => {
      if (url === '/api/schemas') {
        return Promise.resolve({
          ok: true,
          json: async () => ({
            data: [
              {
                name: 'UserProfile',
                fields: {
                  id: { field_type: 'string', writable: false },
                  name: { field_type: 'string', writable: true },
                  email: { field_type: 'string', writable: true }
                }
              },
              {
                name: 'BlogPost',
                fields: {
                  title: { field_type: 'string', writable: true },
                  content: { field_type: 'text', writable: true }
                }
              }
            ]
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
              test_field: { field_type: 'string', writable: true }
            }
          })
        })
      }
      
      if (url === '/api/schema' && fetch.mock.calls[fetch.mock.calls.length - 1][1]?.method === 'POST') {
        return Promise.resolve({
          ok: true,
          json: async () => ({ success: true, message: 'Schema created successfully' })
        })
      }
      
      if (url.startsWith('/api/schema/') && fetch.mock.calls[fetch.mock.calls.length - 1][1]?.method === 'DELETE') {
        return Promise.resolve({
          ok: true,
          json: async () => ({ success: true })
        })
      }
      
      return Promise.reject(new Error(`Unhandled request: ${url}`))
    })
  })

  it('completes full schema management workflow', async () => {
    render(<App />)
    
    // Wait for initial load
    await waitFor(() => {
      expect(screen.getByText('UserProfile')).toBeInTheDocument()
      expect(screen.getByText('BlogPost')).toBeInTheDocument()
    })
    
    // Verify schemas tab is active
    expect(screen.getByText('Schemas')).toHaveClass('text-primary')
    
    // Expand a schema to view details
    await user.click(screen.getByText('UserProfile'))
    
    await waitFor(() => {
      expect(screen.getByText('id')).toBeInTheDocument()
      expect(screen.getByText('name')).toBeInTheDocument()
      expect(screen.getByText('email')).toBeInTheDocument()
    })
    
    // Verify field details are displayed
    expect(screen.getByText('Read-only')).toBeInTheDocument()
    expect(screen.getAllByText('Writable')).toHaveLength(2)
    
    // Collapse the schema
    await user.click(screen.getByText('UserProfile'))
    
    await waitFor(() => {
      expect(screen.queryByText('id')).not.toBeInTheDocument()
    })
  })

  it('loads sample schema successfully', async () => {
    render(<App />)
    
    // Wait for initial load
    await waitFor(() => {
      expect(screen.getByText('UserProfile')).toBeInTheDocument()
    })
    
    // Select a sample schema
    const select = screen.getByRole('combobox')
    await user.selectOptions(select, 'SampleUser')
    
    expect(select.value).toBe('SampleUser')
    
    // Load the sample
    const loadButton = screen.getByText('Load')
    expect(loadButton).not.toBeDisabled()
    
    await user.click(loadButton)
    
    // Verify API calls were made
    await waitFor(() => {
      expect(fetch).toHaveBeenCalledWith('/api/samples/schema/SampleUser')
      expect(fetch).toHaveBeenCalledWith('/api/schema', expect.objectContaining({
        method: 'POST',
        headers: { 'Content-Type': 'application/json' }
      }))
    })
    
    // Verify success result is displayed
    await waitFor(() => {
      expect(screen.getByText(/Schema created successfully/)).toBeInTheDocument()
    })
  })

  it('removes schema successfully', async () => {
    render(<App />)
    
    // Wait for initial load
    await waitFor(() => {
      expect(screen.getByText('UserProfile')).toBeInTheDocument()
    })
    
    // Find and click remove button for UserProfile
    const removeButtons = screen.getAllByText('Remove')
    await user.click(removeButtons[0])
    
    // Verify delete API call was made
    await waitFor(() => {
      expect(fetch).toHaveBeenCalledWith('/api/schema/UserProfile', { method: 'DELETE' })
    })
    
    // Verify schemas are refetched
    await waitFor(() => {
      expect(fetch).toHaveBeenCalledWith('/api/schemas')
    })
  })

  it('navigates between tabs and maintains state', async () => {
    render(<App />)
    
    // Wait for initial load
    await waitFor(() => {
      expect(screen.getByText('UserProfile')).toBeInTheDocument()
    })
    
    // Switch to Query tab
    await user.click(screen.getByText('Query'))
    
    await waitFor(() => {
      expect(screen.getByText('Query')).toHaveClass('text-primary')
      expect(screen.getByText('Schemas')).toHaveClass('text-gray-500')
    })
    
    // Switch to Mutation tab
    await user.click(screen.getByText('Mutation'))
    
    await waitFor(() => {
      expect(screen.getByText('Mutation')).toHaveClass('text-primary')
      expect(screen.getByText('Query')).toHaveClass('text-gray-500')
    })
    
    // Switch to Transforms tab
    const transformsButton = screen.getByRole('button', { name: 'Transforms' })
    await user.click(transformsButton)
    
    await waitFor(() => {
      expect(transformsButton).toHaveClass('text-primary')
    })
    
    // Switch to Dependencies tab
    const dependenciesButton = screen.getByRole('button', { name: 'Dependencies' })
    await user.click(dependenciesButton)
    
    await waitFor(() => {
      expect(dependenciesButton).toHaveClass('text-primary')
    })
    
    // Switch back to Schemas tab
    const schemasButton = screen.getByRole('button', { name: 'Schemas' })
    await user.click(schemasButton)
    
    await waitFor(() => {
      expect(screen.getByText('Schemas')).toHaveClass('text-primary')
      expect(screen.getByText('UserProfile')).toBeInTheDocument()
    })
  })

  it('handles API errors gracefully', async () => {
    // Mock API error for schema fetch, then successful sample schemas fetch
    fetch
      .mockImplementationOnce(() => Promise.reject(new Error('Network error')))
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: ['SampleUser', 'SampleBlog'] })
      })
    
    render(<App />)
    
    // Should still render the UI even with API error
    await waitFor(() => {
      expect(screen.getByText('Schemas')).toBeInTheDocument()
      expect(screen.getByText('Load Sample Schema')).toBeInTheDocument()
    })
    
    // Wait for sample schemas to load
    await waitFor(() => {
      const select = screen.getByRole('combobox')
      expect(select).toBeInTheDocument()
    })
    
    // Verify the UI remains stable even with API errors
    await waitFor(() => {
      expect(screen.getByText('Load Sample Schema')).toBeInTheDocument()
      expect(screen.getByRole('combobox')).toBeInTheDocument()
    })
  })

  it('displays loading states correctly', async () => {
    // Mock delayed response for schemas
    fetch.mockImplementationOnce(() =>
      new Promise(resolve =>
        setTimeout(() => resolve({
          ok: true,
          json: async () => ({ data: [] })
        }), 100)
      )
    )
    
    render(<App />)
    
    // Initial state should show loading behavior
    await waitFor(() => {
      expect(screen.getByText('Schemas')).toBeInTheDocument()
    })
    
    // Verify UI elements are present and functional
    await waitFor(() => {
      expect(screen.getByRole('combobox')).toBeInTheDocument()
      expect(screen.getByText('Load')).toBeInTheDocument()
    })
    
    // Verify load button is disabled when no option is selected
    const loadButton = screen.getByText('Load')
    expect(loadButton).toBeDisabled()
  })

  it('maintains UI consistency across interactions', async () => {
    render(<App />)
    
    // Wait for initial load
    await waitFor(() => {
      expect(screen.getByText('UserProfile')).toBeInTheDocument()
    })
    
    // Verify header and footer are always present
    expect(screen.getByText('DataFold Node')).toBeInTheDocument()
    expect(screen.getByText('Node Active')).toBeInTheDocument()
    
    // Switch tabs and verify layout consistency
    const tabs = ['Query', 'Mutation', 'Transforms', 'Dependencies']
    
    for (const tab of tabs) {
      // Find the button specifically, not just any element with the text
      const tabButton = screen.getByRole('button', { name: tab })
      await user.click(tabButton)
      
      await waitFor(() => {
        expect(tabButton).toHaveClass('text-primary')
      })
      
      // Header and status should still be present
      expect(screen.getByText('DataFold Node')).toBeInTheDocument()
      expect(screen.getByText('Node is running successfully')).toBeInTheDocument()
    }
  })
})