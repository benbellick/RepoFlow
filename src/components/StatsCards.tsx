import type { JSX } from 'react'
import { Card } from './ui/Card'
import { ArrowUpRight, ArrowDownRight, Minus } from 'lucide-react'

interface StatCardProps {
  label: string
  value: string | number
  trend?: 'up' | 'down' | 'neutral'
  trendLabel?: string
  color?: string
}

export const StatCard = ({ label, value, trend, trendLabel, color = 'bg-white' }: StatCardProps): JSX.Element => {
  return (
    <Card className={color}>
      <p className="font-base text-sm uppercase tracking-wider mb-1">{label}</p>
      <h3 className="text-4xl font-heading mb-2">{value}</h3>
      {trend && (
        <div className="flex items-center gap-1 font-heading text-sm">
          {trend === 'up' && <ArrowUpRight size={16} strokeWidth={3} />}
          {trend === 'down' && <ArrowDownRight size={16} strokeWidth={3} />}
          {trend === 'neutral' && <Minus size={16} strokeWidth={3} />}
          <span>{trendLabel}</span>
        </div>
      )}
    </Card>
  )
}
