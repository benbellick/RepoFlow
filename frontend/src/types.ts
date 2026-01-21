/**
 * Represents the direction of a metric trend.
 */
export const TrendDirection = {
  UP: 'up',
  DOWN: 'down',
  NEUTRAL: 'neutral',
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

/**
 * Represents the summary statistics for the latest period.
 */
export interface SummaryMetrics {
  current_opened: number;
  current_merged: number;
  current_spread: number;
  merge_rate: number;
  is_widening: boolean;
}

/**
 * The root response structure for repository metrics from the backend.
 */
export interface RepoMetricsResponse {
  summary: SummaryMetrics;
  time_series: FlowMetrics[];
}

/**
 * Represents a popular repository returned by the backend.
 */
export interface PopularRepo {
  owner: string;
  repo: string;
}
