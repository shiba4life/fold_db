import { render, screen } from '@testing-library/react'
import { describe, it, expect } from 'vitest'
import Header from '../../components/Header'

describe('Header Component', () => {
  it('renders header with correct title', () => {
    render(<Header />)
    
    expect(screen.getByText('DataFold Node')).toBeInTheDocument()
  })

  it('has correct styling classes', () => {
    render(<Header />)
    
    const header = screen.getByRole('banner')
    expect(header).toHaveClass('bg-white', 'border-b', 'border-gray-200', 'py-4', 'px-6', 'shadow-sm')
  })

  it('displays database SVG icon', () => {
    render(<Header />)
    
    const svg = document.querySelector('svg')
    expect(svg).toBeInTheDocument()
    expect(svg).toHaveClass('w-8', 'h-8', 'flex-shrink-0', 'text-primary')
  })

  it('has proper semantic structure', () => {
    render(<Header />)
    
    const header = screen.getByRole('banner')
    expect(header).toBeInTheDocument()
    
    const link = screen.getByRole('link')
    expect(link).toBeInTheDocument()
    expect(link).toHaveAttribute('href', '/')
  })

  it('displays node status indicator', () => {
    render(<Header />)
    
    expect(screen.getByText('Node Active')).toBeInTheDocument()
    
    const statusBadge = screen.getByText('Node Active')
    expect(statusBadge).toHaveClass('inline-flex', 'items-center', 'px-3', 'py-1', 'rounded-full', 'text-sm', 'font-medium', 'bg-green-100', 'text-green-800')
  })

  it('has responsive layout classes', () => {
    render(<Header />)
    
    const container = screen.getByRole('banner').firstChild
    expect(container).toHaveClass('max-w-7xl', 'mx-auto', 'flex', 'items-center', 'justify-between')
  })

  it('title link has hover effects', () => {
    render(<Header />)
    
    const link = screen.getByRole('link')
    expect(link).toHaveClass('flex', 'items-center', 'gap-3', 'text-primary', 'hover:text-primary/90', 'transition-colors')
  })

  it('status indicator has green dot', () => {
    render(<Header />)
    
    const statusContainer = screen.getByText('Node Active').parentElement
    const greenDot = statusContainer.querySelector('.bg-green-500')
    expect(greenDot).toBeInTheDocument()
    expect(greenDot).toHaveClass('w-2', 'h-2', 'rounded-full', 'bg-green-500', 'mr-2')
  })

  it('title has correct typography classes', () => {
    render(<Header />)
    
    const title = screen.getByText('DataFold Node')
    expect(title).toHaveClass('text-xl', 'font-semibold')
  })
})