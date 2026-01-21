import type { JSX } from 'react'
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Legend,
} from 'recharts'
import type { FlowMetrics } from '../types'
import { Card } from './ui/Card'

/**
 * Props for the FlowChart component.
 */
interface FlowChartProps {
  /** Array of flow metrics to visualize. */
  data: FlowMetrics[]
}

/**
 * A line chart visualization of PR flow rates over time.
 * Uses Recharts to display "PRs Opened" and "PRs Merged" series.
 */
export const FlowChart = ({ data }: FlowChartProps): JSX.Element => {
  return (
    <Card className="w-full h-[400px]">
      <h2 className="text-2xl mb-4">PR Flow Rates</h2>
      <ResponsiveContainer width="100%" height="90%">
        <LineChart data={data}>
          <CartesianGrid strokeDasharray="3 3" stroke="#000" vertical={false} />
          <XAxis
            dataKey="date"
            stroke="#000"
            tick={{ fill: '#000', fontWeight: 'bold' }}
            axisLine={{ strokeWidth: 2 }}
          />
          <YAxis
            stroke="#000"
            tick={{ fill: '#000', fontWeight: 'bold' }}
            axisLine={{ strokeWidth: 2 }}
          />
          <Tooltip
            contentStyle={{
              backgroundColor: '#fff',
              border: '4px solid #000',
              borderRadius: '0px',
              fontWeight: 'bold',
            }}
          />
          <Legend />
          <Line
            type="monotone"
            dataKey="opened"
            stroke="#2563eb"
            strokeWidth={4}
            dot={{ r: 4, strokeWidth: 2 }}
            activeDot={{ r: 8 }}
            name="PRs Opened"
          />
          <Line
            type="monotone"
            dataKey="merged"
            stroke="#db2777"
            strokeWidth={4}
            dot={{ r: 4, strokeWidth: 2 }}
            activeDot={{ r: 8 }}
            name="PRs Merged"
          />
        </LineChart>
      </ResponsiveContainer>
    </Card>
  )
}
