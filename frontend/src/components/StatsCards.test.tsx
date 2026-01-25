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

  it('renders trend information when provided', () => {
    render(
      <StatCard
        label="With Trend"
        value="456"
        trend={TrendDirection.UP}
        trendLabel="+10%"
      />
    )
    expect(screen.getByText('+10%')).toBeTruthy()
  })

  it('shows tooltip description on hover (structure check)', () => {
    // Note: Testing actual hover/tooltip visibility often requires more complex setup or userEvent.
    // Here we just check if the description text is present in the document (hidden or not).
    render(
      <StatCard
        label="With Tooltip"
        value="789"
        description="This is a description"
      />
    )
    // The description might be hidden by CSS (display: none), but it should exist in the DOM.
    // getByText might fail if it's strictly checking visibility, so we use getAllByText or check existence.
    // However, the description is in the DOM.
    expect(screen.getByText('This is a description')).toBeTruthy()
  })
})
