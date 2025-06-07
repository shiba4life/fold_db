import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'
import LogSidebar from '../../components/LogSidebar'

describe('LogSidebar', () => {
  it('copies logs to clipboard', async () => {
    fetch.mockResolvedValueOnce({ ok: true, json: async () => ['line1', 'line2'] })
    const es = { onmessage: null, close: vi.fn() }
    global.EventSource.mockImplementation(() => es)
    const writeText = vi.fn()
    Object.assign(navigator, { clipboard: { writeText } })

    render(<LogSidebar />)
    await screen.findByText('line1')
    fireEvent.click(screen.getByText('Copy'))
    expect(writeText).toHaveBeenCalledWith('line1\nline2')
  })
})
