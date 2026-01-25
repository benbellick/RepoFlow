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
    render(<StatCard label="With Trend" value="456" trend={TrendDirection.UP} trendLabel="+10%" />)
    expect(screen.getByText('+10%')).toBeTruthy()
  })

  it('shows tooltip description on hover (structure check)', () => {
    render(<StatCard label="With Tooltip" value="789" description="This is a description" />)
    expect(screen.getByText('This is a description')).toBeTruthy()
  })
})
