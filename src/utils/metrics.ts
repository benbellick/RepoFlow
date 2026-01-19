import type { GitHubPR } from './github';

/**
 * Represents the calculated metrics for a specific date.
 */
export interface FlowMetrics {
  date: string;
  opened: number;
  merged: number;
  spread: number;
}

/**
 * Calculates rolling window metrics from raw GitHub PR data.
 * 
 * @param prs - Array of raw GitHubPR objects.
 * @param daysToDisplay - Number of days to include in the output array (default: 30).
 * @param windowSize - The size of the rolling window in days (default: 30).
 * @returns An array of FlowMetrics, one for each day.
 */
export const calculateMetrics = (
  prs: GitHubPR[], 
  daysToDisplay: number = 30, 
  windowSize: number = 30
): FlowMetrics[] => {
  const data: FlowMetrics[] = [];
  const now = new Date();
  
  for (let i = daysToDisplay; i >= 0; i--) {
    const targetDate = new Date(now);
    targetDate.setHours(23, 59, 59, 999);
    targetDate.setDate(targetDate.getDate() - i);
    
    const windowStart = new Date(targetDate);
    windowStart.setDate(windowStart.getDate() - windowSize);
    
    const openedInWindow = prs.filter(pr => {
      const created = new Date(pr.created_at);
      return created >= windowStart && created <= targetDate;
    }).length;
    
    const mergedInWindow = prs.filter(pr => {
      if (!pr.merged_at) return false;
      const merged = new Date(pr.merged_at);
      return merged >= windowStart && merged <= targetDate;
    }).length;
    
    data.push({
      date: targetDate.toISOString().split('T')[0],
      opened: openedInWindow,
      merged: mergedInWindow,
      spread: openedInWindow - mergedInWindow,
    });
  }
  
  return data;
};

/**
 * Generates dummy flow data for testing and preview purposes.
 * 
 * @param days - Number of days to generate data for (default: 30).
 * @returns An array of dummy FlowMetrics.
 */
export const generateDummyData = (days: number = 30): FlowMetrics[] => {
  const data: FlowMetrics[] = [];
  const now = new Date();
  
  for (let i = days; i >= 0; i--) {
    const date = new Date(now);
    date.setDate(date.getDate() - i);
    
    const opened = Math.floor(Math.random() * 20) + 10;
    const merged = Math.floor(Math.random() * 15) + 5;
    
    data.push({
      date: date.toISOString().split('T')[0],
      opened,
      merged,
      spread: opened - merged,
    });
  }
  
  return data;
};