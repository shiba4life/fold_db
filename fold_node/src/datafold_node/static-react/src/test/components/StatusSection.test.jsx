import { render, screen } from '@testing-library/react'
import { describe, it, expect } from 'vitest'
import StatusSection from '../../components/StatusSection'

describe('StatusSection Component', () => {
  it('renders status message correctly', () => {
    render(<StatusSection />)
    
    expect(screen.getByText('Node is running successfully')).toBeInTheDocument()
    expect(screen.getByText('Active and healthy')).toBeInTheDocument()
  })

  it('has correct container styling', () => {
    render(<StatusSection />)
    
    const container = screen.getByText('Node is running successfully').closest('div').parentElement
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
})