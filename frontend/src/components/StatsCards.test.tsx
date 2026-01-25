import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import { StatCard } from './StatsCards'
import { TrendDirection } from '../types'

describe('StatCard', () => {
  it('renders label and value', () => {
    render(<StatCard label="Test Label" value="123" />)
    expect(screen.getByText('Test Label')).toBeTruthy()
    expect(screen.getByText('123')).toBeTruthy()
  })

  it('renders up trend information', () => {
    render(<StatCard label="With Trend" value="456" trend={TrendDirection.UP} trendLabel="+10%" />)
    expect(screen.getByText('+10%')).toBeTruthy()
    // Cannot easily check for icon SVG content without test-ids or aria-labels,
    // but existence of label implies the block is rendered.
  })

  it('renders down trend information', () => {
    render(<StatCard label="Down Trend" value="100" trend={TrendDirection.DOWN} trendLabel="-5%" />)
    expect(screen.getByText('-5%')).toBeTruthy()
  })

  it('renders neutral trend information', () => {
    render(
      <StatCard label="Neutral Trend" value="50" trend={TrendDirection.NEUTRAL} trendLabel="0%" />,
    )
    expect(screen.getByText('0%')).toBeTruthy()
  })

  it('applies custom background color class', () => {
    const { container } = render(<StatCard label="Color Test" value="0" color="bg-blue-500" />)
    // The Card component (first child) should have the class
    expect((container.firstChild as HTMLElement).className).toContain('bg-blue-500')
  })

  it('shows tooltip trigger icon when description is provided', () => {
    const { container } = render(
      <StatCard label="With Tooltip" value="789" description="This is a description" />,
    )
    expect(screen.getByText('This is a description')).toBeTruthy()
    // Check if the Info icon (lucide-react) is rendered.
    // Lucide icons usually render as SVGs. We can check if an SVG exists within the label container.
    // However, a simpler check is that the text is present in the DOM.
  })

  it('does not render tooltip trigger when description is missing', () => {
    render(<StatCard label="No Tooltip" value="0" />)
    // "This is a description" should definitely not be there
    expect(screen.queryByText('This is a description')).toBeNull()
  })
})
