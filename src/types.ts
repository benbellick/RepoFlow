/**
 * Represents the direction of a metric trend.
 */
export const TrendDirection = {
  UP: 'up',
  DOWN: 'down',
  NEUTRAL: 'neutral'
} as const;

export type TrendDirection = (typeof TrendDirection)[keyof typeof TrendDirection];