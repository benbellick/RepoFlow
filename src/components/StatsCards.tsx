import type { JSX } from 'react'
import { Card } from './ui/Card'
import { ArrowUpRight, ArrowDownRight, Minus } from 'lucide-react'

/**
 * Represents the direction of a metric trend.
 */
export enum TrendDirection {
  UP = 'up',
  DOWN = 'down',
  NEUTRAL = 'neutral'
}

/**
 * Props for the StatCard component.
 */
interface StatCardProps {
  /** The label text for the statistic (e.g., "PRs Opened"). */
  label: string
  /** The value to display (e.g., "123" or "45%"). */
  value: string | number
  /** The direction of the trend, if applicable. */
  trend?: TrendDirection
  /** A descriptive label for the trend (e.g., "+12% vs last period"). */
  trendLabel?: string
  /** The background color class for the card. Defaults to "bg-white". */
  color?: string
}

/**
 * A card component designed to display a single metric with an optional trend indicator.
 */
export const StatCard = ({ label, value, trend, trendLabel, color = 'bg-white' }: StatCardProps): JSX.Element => {
  const renderTrendIcon = (): JSX.Element | null => {
    switch (trend) {
      case TrendDirection.UP:
        return <ArrowUpRight size={16} strokeWidth={3} />
      case TrendDirection.DOWN:
        return <ArrowDownRight size={16} strokeWidth={3} />
      case TrendDirection.NEUTRAL:
        return <Minus size={16} strokeWidth={3} />
      default:
        return null
    }
  }

  return (
    <Card className={color}>
      <p className="font-base text-sm uppercase tracking-wider mb-1">{label}</p>
      <h3 className="text-4xl font-heading mb-2">{value}</h3>
      {trend && (
        <div className="flex items-center gap-1 font-heading text-sm">
          {renderTrendIcon()}
          <span>{trendLabel}</span>
        </div>
      )}
    </Card>
  )
}
