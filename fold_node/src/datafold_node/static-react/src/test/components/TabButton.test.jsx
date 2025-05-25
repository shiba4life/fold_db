import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'
import TabButton from '../../components/TabButton'

describe('TabButton Component', () => {
  it('renders label', () => {
    render(
      <TabButton label="Schemas" tabKey="schemas" activeTab="schemas" onChange={() => {}} />
    )
    expect(screen.getByText('Schemas')).toBeInTheDocument()
  })

  it('calls onChange with tabKey when clicked', () => {
    const onChange = vi.fn()
    render(
      <TabButton label="Query" tabKey="query" activeTab="schemas" onChange={onChange} />
    )
    fireEvent.click(screen.getByText('Query'))
    expect(onChange).toHaveBeenCalledWith('query')
  })

  it('applies active styles when active', () => {
    render(
      <TabButton label="Mutation" tabKey="mutation" activeTab="mutation" onChange={() => {}} />
    )
    const btn = screen.getByText('Mutation')
    expect(btn).toHaveClass('text-primary')
    expect(btn).toHaveClass('border-b-2')
  })

  it('applies inactive styles when not active', () => {
    render(
      <TabButton label="Deps" tabKey="dependencies" activeTab="schemas" onChange={() => {}} />
    )
    const btn = screen.getByText('Deps')
    expect(btn).toHaveClass('text-gray-500')
  })
})
