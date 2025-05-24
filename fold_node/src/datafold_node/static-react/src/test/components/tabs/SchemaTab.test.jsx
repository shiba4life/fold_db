import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { describe, it, expect, beforeEach, vi } from 'vitest'
import SchemaTab from '../../../components/tabs/SchemaTab'

describe('SchemaTab Component', () => {
  const mockSchemas = [
    {
      name: 'UserProfile',
      fields: {
        id: {
          field_type: 'string',
          writable: false,
          ref_atom_uuid: 'uuid-123'
        },
        name: {
          field_type: 'string',
          writable: true,
          transform: { name: 'capitalize' }
        },
        email: {
          field_type: 'string',
          writable: true
        }
      }
    },
    {
      name: 'BlogPost',
      fields: {
        title: {
          field_type: 'string',
          writable: true
        },
        content: {
          field_type: 'text',
          writable: true
        }
      }
    }
  ]

  const mockProps = {
    schemas: mockSchemas,
    onResult: vi.fn(),
    onSchemaUpdated: vi.fn()
  }

  beforeEach(() => {
    vi.clearAllMocks()
    
    // Mock sample schemas API
    fetch.mockResolvedValue({
      ok: true,
      json: async () => ({
        data: ['SampleSchema1', 'SampleSchema2', 'SampleSchema3']
      })
    })
  })

  it('renders schema list correctly', async () => {
    render(<SchemaTab {...mockProps} />)
    
    await waitFor(() => {
      expect(screen.getByText('UserProfile')).toBeInTheDocument()
      expect(screen.getByText('BlogPost')).toBeInTheDocument()
    })
    
    expect(screen.getByText('(3 fields)')).toBeInTheDocument()
    expect(screen.getByText('(2 fields)')).toBeInTheDocument()
  })

  it('fetches sample schemas on mount', async () => {
    render(<SchemaTab {...mockProps} />)
    
    await waitFor(() => {
      expect(fetch).toHaveBeenCalledWith('/api/samples/schemas')
    })
  })

  it('displays sample schema dropdown', async () => {
    render(<SchemaTab {...mockProps} />)
    
    await waitFor(() => {
      expect(screen.getByText('Select a sample...')).toBeInTheDocument()
    })
    
    const select = screen.getByRole('combobox')
    expect(select).toBeInTheDocument()
  })

  it('populates sample schema options', async () => {
    render(<SchemaTab {...mockProps} />)
    
    await waitFor(() => {
      const select = screen.getByRole('combobox')
      expect(select).toBeInTheDocument()
    })
    
    // Check if options are populated (they should be in the DOM)
    await waitFor(() => {
      expect(screen.getByText('SampleSchema1')).toBeInTheDocument()
      expect(screen.getByText('SampleSchema2')).toBeInTheDocument()
      expect(screen.getByText('SampleSchema3')).toBeInTheDocument()
    })
  })

  it('expands and collapses schema details', async () => {
    render(<SchemaTab {...mockProps} />)
    
    await waitFor(() => {
      expect(screen.getByText('UserProfile')).toBeInTheDocument()
    })
    
    // Initially collapsed - fields should not be visible
    expect(screen.queryByText('id')).not.toBeInTheDocument()
    expect(screen.queryByText('name')).not.toBeInTheDocument()
    
    // Click to expand
    fireEvent.click(screen.getByText('UserProfile'))
    
    // Fields should now be visible
    expect(screen.getByText('id')).toBeInTheDocument()
    expect(screen.getByText('name')).toBeInTheDocument()
    expect(screen.getByText('email')).toBeInTheDocument()
    
    // Click to collapse
    fireEvent.click(screen.getByText('UserProfile'))
    
    // Fields should be hidden again
    expect(screen.queryByText('id')).not.toBeInTheDocument()
    expect(screen.queryByText('name')).not.toBeInTheDocument()
  })

  it('displays field information correctly', async () => {
    render(<SchemaTab {...mockProps} />)
    
    await waitFor(() => {
      expect(screen.getByText('UserProfile')).toBeInTheDocument()
    })
    
    // Expand schema
    fireEvent.click(screen.getByText('UserProfile'))
    
    // Check field types
    expect(screen.getAllByText('string')).toHaveLength(3)
    
    // Check writable status
    expect(screen.getByText('Read-only')).toBeInTheDocument()
    expect(screen.getAllByText('Writable')).toHaveLength(2)
    
    // Check transform information
    expect(screen.getByText('capitalize')).toBeInTheDocument()
    
    // Check UUID display
    expect(screen.getByText('uuid-123')).toBeInTheDocument()
  })

  it('handles schema removal', async () => {
    const testProps = {
      ...mockProps,
      onSchemaUpdated: vi.fn()
    }
    
    fetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({ data: [] })
    }).mockResolvedValueOnce({
      ok: true
    })
    
    render(<SchemaTab {...testProps} />)
    
    await waitFor(() => {
      expect(screen.getByText('UserProfile')).toBeInTheDocument()
    })
    
    // Find and click the first remove button
    const removeButtons = screen.getAllByText('Remove')
    fireEvent.click(removeButtons[0])
    
    await waitFor(() => {
      expect(fetch).toHaveBeenCalledWith('/api/schema/UserProfile', { method: 'DELETE' })
    })
    
    await waitFor(() => {
      expect(testProps.onSchemaUpdated).toHaveBeenCalled()
    })
  })


  it('disables load button when no sample selected', async () => {
    render(<SchemaTab {...mockProps} />)
    
    await waitFor(() => {
      expect(screen.getByText('Load')).toBeInTheDocument()
    })
    
    const loadButton = screen.getByText('Load')
    expect(loadButton).toBeDisabled()
    expect(loadButton).toHaveClass('bg-gray-300', 'cursor-not-allowed')
  })


  it('prevents event propagation on remove button click', async () => {
    fetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({ data: [] })
    }).mockResolvedValueOnce({
      ok: true
    })
    
    render(<SchemaTab {...mockProps} />)
    
    await waitFor(() => {
      expect(screen.getByText('UserProfile')).toBeInTheDocument()
    })
    
    // Schema should be collapsed initially
    expect(screen.queryByText('id')).not.toBeInTheDocument()
    
    // Click remove button (should not expand schema)
    const removeButtons = screen.getAllByText('Remove')
    fireEvent.click(removeButtons[0])
    
    // Schema should still be collapsed
    expect(screen.queryByText('id')).not.toBeInTheDocument()
  })
})