import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { describe, it, expect, beforeEach, vi } from 'vitest'
import App from '../App'

// Mock the components to focus on App logic
vi.mock('../components/Header', () => ({
  default: () => <div data-testid="header">Header</div>
}))

vi.mock('../components/Footer', () => ({
  default: () => <div data-testid="footer">Footer</div>
}))

vi.mock('../components/StatusSection', () => ({
  default: () => <div data-testid="status-section">Status</div>
}))

vi.mock('../components/ResultsSection', () => ({
  default: ({ results }) => (
    <div data-testid="results-section">
      Results: {JSON.stringify(results)}
    </div>
  )
}))

vi.mock('../components/LogSidebar', () => ({
  default: () => <div data-testid="log-sidebar">Log Sidebar</div>
}))

vi.mock('../components/tabs/SchemaTab', () => ({
  default: ({ schemas, onResult, onSchemaUpdated }) => (
    <div data-testid="schema-tab">
      Schema Tab - {schemas.length} schemas
      <button onClick={() => onResult({ success: true })}>Trigger Result</button>
      <button onClick={() => onSchemaUpdated()}>Update Schema</button>
    </div>
  )
}))

vi.mock('../components/tabs/QueryTab', () => ({
  default: ({ schemas, onResult }) => (
    <div data-testid="query-tab">
      Query Tab - {schemas.length} schemas
      <button onClick={() => onResult({ query: 'test' })}>Execute Query</button>
    </div>
  )
}))

vi.mock('../components/tabs/MutationTab', () => ({
  default: ({ schemas, onResult }) => (
    <div data-testid="mutation-tab">
      Mutation Tab - {schemas.length} schemas
      <button onClick={() => onResult({ mutation: 'test' })}>Execute Mutation</button>
    </div>
  )
}))

vi.mock('../components/tabs/TransformsTab', () => ({
  default: ({ schemas, onResult }) => (
    <div data-testid="transforms-tab">
      Transforms Tab - {schemas.length} schemas
      <button onClick={() => onResult({ transform: 'test' })}>Execute Transform</button>
    </div>
  )
}))

vi.mock('../components/tabs/SchemaDependenciesTab', () => ({
  default: ({ schemas }) => (
    <div data-testid="dependencies-tab">
      Dependencies Tab - {schemas.length} schemas
    </div>
  )
}))

describe('App Component', () => {
  beforeEach(() => {
    // Mock successful API response for schemas
    fetch.mockResolvedValue({
      ok: true,
      json: async () => ({
        data: [
          { name: 'TestSchema1', state: 'Loaded', fields: {} },
          { name: 'TestSchema2', state: 'Loaded', fields: {} }
        ]
      })
    })
  })

  it('renders main layout components', async () => {
    render(<App />)
    
    expect(screen.getByTestId('header')).toBeInTheDocument()
    expect(screen.getByTestId('footer')).toBeInTheDocument()
    expect(screen.getByTestId('status-section')).toBeInTheDocument()
    expect(screen.getByTestId('log-sidebar')).toBeInTheDocument()
  })

  it('renders all navigation tabs', async () => {
    render(<App />)
    
    expect(screen.getByText('Schemas')).toBeInTheDocument()
    expect(screen.getByText('Query')).toBeInTheDocument()
    expect(screen.getByText('Mutation')).toBeInTheDocument()
    expect(screen.getByText('Transforms')).toBeInTheDocument()
    expect(screen.getByText('Dependencies')).toBeInTheDocument()
  })

  it('starts with schemas tab active', async () => {
    render(<App />)
    
    await waitFor(() => {
      expect(screen.getByTestId('schema-tab')).toBeInTheDocument()
    })
    
    const schemasButton = screen.getByText('Schemas')
    expect(schemasButton).toHaveClass('text-primary', 'border-b-2', 'border-primary')
  })

  it('fetches schemas on mount', async () => {
    render(<App />)
    
    await waitFor(() => {
      expect(fetch).toHaveBeenCalledWith('/api/schemas')
    })
    
    await waitFor(() => {
      expect(screen.getByText('Schema Tab - 2 schemas')).toBeInTheDocument()
    })
  })

  it('handles schema fetch error gracefully', async () => {
    fetch.mockRejectedValue(new Error('Network error'))
    
    render(<App />)
    
    await waitFor(() => {
      expect(fetch).toHaveBeenCalledWith('/api/schemas')
    })
    
    // Should still render with empty schemas
    await waitFor(() => {
      expect(screen.getByText('Schema Tab - 0 schemas')).toBeInTheDocument()
    })
  })

  it('switches tabs correctly', async () => {
    render(<App />)
    
    // Wait for initial load
    await waitFor(() => {
      expect(screen.getByTestId('schema-tab')).toBeInTheDocument()
    })
    
    // Click Query tab
    fireEvent.click(screen.getByText('Query'))
    expect(screen.getByTestId('query-tab')).toBeInTheDocument()
    expect(screen.queryByTestId('schema-tab')).not.toBeInTheDocument()
    
    // Click Mutation tab
    fireEvent.click(screen.getByText('Mutation'))
    expect(screen.getByTestId('mutation-tab')).toBeInTheDocument()
    expect(screen.queryByTestId('query-tab')).not.toBeInTheDocument()
    
    // Click Transforms tab
    fireEvent.click(screen.getByText('Transforms'))
    expect(screen.getByTestId('transforms-tab')).toBeInTheDocument()
    
    // Click Dependencies tab
    fireEvent.click(screen.getByText('Dependencies'))
    expect(screen.getByTestId('dependencies-tab')).toBeInTheDocument()
  })

  it('updates tab styling when switching', async () => {
    render(<App />)
    
    await waitFor(() => {
      expect(screen.getByTestId('schema-tab')).toBeInTheDocument()
    })
    
    const queryButton = screen.getByText('Query')
    const schemasButton = screen.getByText('Schemas')
    
    // Initially schemas is active
    expect(schemasButton).toHaveClass('text-primary')
    expect(queryButton).toHaveClass('text-gray-500')
    
    // Click query tab
    fireEvent.click(queryButton)
    
    expect(queryButton).toHaveClass('text-primary')
    expect(schemasButton).toHaveClass('text-gray-500')
  })

  it('clears results when switching tabs', async () => {
    render(<App />)
    
    await waitFor(() => {
      expect(screen.getByTestId('schema-tab')).toBeInTheDocument()
    })
    
    // Trigger a result in schema tab
    fireEvent.click(screen.getByText('Trigger Result'))
    expect(screen.getByTestId('results-section')).toBeInTheDocument()
    
    // Switch to query tab
    fireEvent.click(screen.getByText('Query'))
    
    // Results should be cleared
    expect(screen.queryByTestId('results-section')).not.toBeInTheDocument()
  })

  it('displays results when operation completes', async () => {
    render(<App />)
    
    await waitFor(() => {
      expect(screen.getByTestId('schema-tab')).toBeInTheDocument()
    })
    
    // Trigger a result
    fireEvent.click(screen.getByText('Trigger Result'))
    
    expect(screen.getByTestId('results-section')).toBeInTheDocument()
    expect(screen.getByText('Results: {"success":true}')).toBeInTheDocument()
  })

  it('refetches schemas when schema is updated', async () => {
    render(<App />)
    
    await waitFor(() => {
      expect(screen.getByTestId('schema-tab')).toBeInTheDocument()
    })
    
    // Clear the initial fetch call
    fetch.mockClear()
    
    // Mock updated response
    fetch.mockResolvedValue({
      ok: true,
      json: async () => ({
        data: [
          { name: 'UpdatedSchema', fields: {} }
        ]
      })
    })
    
    // Trigger schema update
    fireEvent.click(screen.getByText('Update Schema'))
    
    await waitFor(() => {
      expect(fetch).toHaveBeenCalledWith('/api/schemas')
    })
  })

  it('passes correct props to tab components', async () => {
    render(<App />)
    
    await waitFor(() => {
      expect(screen.getByText('Schema Tab - 2 schemas')).toBeInTheDocument()
    })
    
    // Switch to query tab
    fireEvent.click(screen.getByText('Query'))
    expect(screen.getByText('Query Tab - 2 schemas')).toBeInTheDocument()
    
    // Switch to mutation tab
    fireEvent.click(screen.getByText('Mutation'))
    expect(screen.getByText('Mutation Tab - 2 schemas')).toBeInTheDocument()
    
    // Switch to transforms tab
    fireEvent.click(screen.getByText('Transforms'))
    expect(screen.getByText('Transforms Tab - 2 schemas')).toBeInTheDocument()
    
    // Switch to dependencies tab
    fireEvent.click(screen.getByText('Dependencies'))
    expect(screen.getByText('Dependencies Tab - 2 schemas')).toBeInTheDocument()
  })

  it('handles results from different tabs', async () => {
    render(<App />)
    
    // Test query tab result
    fireEvent.click(screen.getByText('Query'))
    await waitFor(() => {
      expect(screen.getByTestId('query-tab')).toBeInTheDocument()
    })
    
    fireEvent.click(screen.getByText('Execute Query'))
    expect(screen.getByText('Results: {"query":"test"}')).toBeInTheDocument()
    
    // Test mutation tab result
    fireEvent.click(screen.getByText('Mutation'))
    await waitFor(() => {
      expect(screen.getByTestId('mutation-tab')).toBeInTheDocument()
    })
    
    fireEvent.click(screen.getByText('Execute Mutation'))
    expect(screen.getByText('Results: {"mutation":"test"}')).toBeInTheDocument()
    
    // Test transforms tab result
    fireEvent.click(screen.getByText('Transforms'))
    await waitFor(() => {
      expect(screen.getByTestId('transforms-tab')).toBeInTheDocument()
    })
    
    fireEvent.click(screen.getByText('Execute Transform'))
    expect(screen.getByText('Results: {"transform":"test"}')).toBeInTheDocument()
  })

  it('filters schemas by loaded state', async () => {
    fetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        data: [
          { name: 'LoadedSchema', state: 'Loaded', fields: {} },
          { name: 'OtherSchema', state: 'Unloaded', fields: {} }
        ]
      })
    })

    render(<App />)

    await waitFor(() => {
      expect(screen.getByText('Schema Tab - 1 schemas')).toBeInTheDocument()
    })
  })
})