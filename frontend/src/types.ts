/**
 * Represents the direction of a metric trend.
 */
export const TrendDirection = {
  UP: 'up',
  DOWN: 'down',
  NEUTRAL: 'neutral'
} as const;

export type TrendDirection = (typeof TrendDirection)[keyof typeof TrendDirection];

/**
 * Represents the calculated metrics for a specific date returned by the backend.
 */
export interface FlowMetrics {
  date: string;
  opened: number;
  merged: number;
  spread: number;
}
