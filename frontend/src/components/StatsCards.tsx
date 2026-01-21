import type { JSX } from 'react'
import { Card } from './ui/Card'
import { ArrowUpRight, ArrowDownRight, Minus, Info } from 'lucide-react'
import { TrendDirection } from '../types'

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
  /** Optional tooltip text to describe the metric. */
  description?: string
}

/**
 * A card component designed to display a single metric with an optional trend indicator.
 */
export const StatCard = ({
  label,
  value,
  trend,
  trendLabel,
  color = 'bg-white',
  description,
}: StatCardProps): JSX.Element => {
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
    <Card className={`${color} relative group`}>
      <div className="flex justify-between items-start mb-1">
        <p className="font-base text-sm uppercase tracking-wider">{label}</p>
        {description && (
          <div className="relative">
            <Info
              size={16}
              className="text-black/50 cursor-help hover:text-black transition-colors"
            />
            <div className="absolute bottom-full right-0 mb-2 w-48 p-2 bg-black text-white text-xs font-base hidden group-hover:block z-10 shadow-[4px_4px_0px_0px_rgba(255,255,255,1)] border-2 border-white">
              {description}
            </div>
          </div>
        )}
      </div>
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
