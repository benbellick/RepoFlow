use crate::github::GitHubPR;
use chrono::{DateTime, Datelike, Duration, TimeZone, Utc};
use serde::Serialize;

const END_OF_DAY_HOUR: u32 = 23;
const END_OF_DAY_MIN: u32 = 59;
const END_OF_DAY_SEC: u32 = 59;

/// The root response structure for repository metrics.
#[derive(Debug, Serialize, Clone)]
pub struct RepoMetricsResponse {
    /// The calculated summary statistics for the latest period.
    pub summary: SummaryMetrics,
    /// The day-by-day time series data.
    pub time_series: Vec<FlowMetricsResponse>,
}

/// Calculated summary statistics for the latest data point.
#[derive(Debug, Serialize, Clone, Default)]
pub struct SummaryMetrics {
    /// Number of PRs opened in the current rolling window.
    pub current_opened: usize,
    /// Number of PRs merged in the current rolling window.
    pub current_merged: usize,
    /// The current difference between opened and merged PRs.
    pub current_spread: i64,
    /// The percentage of opened PRs that were merged.
    pub merge_rate: u32,
    /// Whether the spread is widening compared to the previous period.
    pub is_widening: bool,
}

/// A single data point in the flow metrics time series.
#[derive(Debug, Serialize, Clone, Default)]
pub struct FlowMetricsResponse {
    /// The date for which the metrics were calculated (YYYY-MM-DD).
    pub date: String,
    /// Number of PRs opened within the rolling window.
    pub opened: usize,
    /// Number of PRs merged within the rolling window.
    pub merged: usize,
    /// The difference between opened and merged PRs.
    pub spread: i64,
}

/// Calculates rolling window metrics from a list of Pull Requests.
///
/// # Arguments
/// * `prs` - The list of PRs to analyze.
/// * `days_to_display` - How many days of history to include in the response.
/// * `window_size` - The size of the rolling window (e.g., 30 days).
/// * `now` - The current time used as the reference point for calculations.
pub fn calculate_metrics(
    prs: &[GitHubPR],
    days_to_display: Duration,
    window_size: Duration,
    now: DateTime<Utc>,
) -> RepoMetricsResponse {
    let days_to_display_count = days_to_display.num_days();
    let window_size_days = window_size.num_days();

    // Determine the date range we need to cover.
    // We display metrics for [now - days_to_display, now].
    // Each data point sums the previous `window_size` days.
    // So we need daily buckets starting from `now - days_to_display - window_size`.

    let latest_date = now.date_naive();
    let oldest_display_date = latest_date - Duration::days(days_to_display_count);
    let oldest_data_date = oldest_display_date - Duration::days(window_size_days);

    // Total days to track in buckets.
    // Example: latest = 10, oldest_data = 0. Range [0, 10]. Size 11.
    let total_days = (latest_date - oldest_data_date).num_days() + 1;
    let total_days_usize = total_days as usize;

    // Use a flat vector for O(1) access by day index.
    let mut daily_opened = vec![0usize; total_days_usize];
    let mut daily_merged = vec![0usize; total_days_usize];

    // Single pass to bucket all PRs
    for pr in prs {
        let created_date = pr.created_at.date_naive();
        if created_date >= oldest_data_date && created_date <= latest_date {
            let idx = (created_date - oldest_data_date).num_days() as usize;
            if idx < total_days_usize {
                daily_opened[idx] += 1;
            }
        }

        if let Some(merged_at) = pr.merged_at {
            let merged_date = merged_at.date_naive();
            if merged_date >= oldest_data_date && merged_date <= latest_date {
                let idx = (merged_date - oldest_data_date).num_days() as usize;
                if idx < total_days_usize {
                    daily_merged[idx] += 1;
                }
            }
        }
    }

    // Compute prefix sums for O(1) range queries
    // prefix[i] = sum(bucket[0]...bucket[i])
    let mut prefix_opened = vec![0usize; total_days_usize];
    let mut prefix_merged = vec![0usize; total_days_usize];

    if total_days_usize > 0 {
        prefix_opened[0] = daily_opened[0];
        prefix_merged[0] = daily_merged[0];

        for i in 1..total_days_usize {
            prefix_opened[i] = prefix_opened[i - 1] + daily_opened[i];
            prefix_merged[i] = prefix_merged[i - 1] + daily_merged[i];
        }
    }

    let time_series: Vec<FlowMetricsResponse> = (0..=days_to_display_count)
        .rev()
        .map(|i| {
            let date = now - Duration::days(i);
            let date_naive = date.date_naive();

            // Map the target date to our bucket index
            let end_idx = (date_naive - oldest_data_date).num_days() as usize;

            // We want the sum of the last `window_size_days` buckets ending at `end_idx`.
            // Range: (end_idx - window_size, end_idx] (indices).
            // Sum = prefix[end_idx] - prefix[end_idx - window_size].
            // If window_size is 30, we sum 30 buckets: end_idx, end_idx-1, ..., end_idx-29.
            // This requires end_idx >= 29 (if 0-based).
            // Our buffer starts `window_size` days before the first display date.
            // So end_idx should always be >= window_size_days.

            let opened = if end_idx >= window_size_days as usize {
                prefix_opened[end_idx] - prefix_opened[end_idx - window_size_days as usize]
            } else {
                // Fallback for safety, though theoretically unreachable with valid dates
                prefix_opened[end_idx]
            };

            let merged = if end_idx >= window_size_days as usize {
                prefix_merged[end_idx] - prefix_merged[end_idx - window_size_days as usize]
            } else {
                prefix_merged[end_idx]
            };

            // We set the time to the end of the day to match the original behavior
            let target_date = Utc
                .with_ymd_and_hms(
                    date.year(),
                    date.month(),
                    date.day(),
                    END_OF_DAY_HOUR,
                    END_OF_DAY_MIN,
                    END_OF_DAY_SEC,
                )
                .unwrap();

            FlowMetricsResponse {
                date: target_date.format("%Y-%m-%d").to_string(),
                opened,
                merged,
                spread: opened as i64 - merged as i64,
            }
        })
        .collect();

    let summary = calculate_summary(&time_series);

    RepoMetricsResponse {
        summary,
        time_series,
    }
}

/// Calculates the summary metrics based on the generated time series.
fn calculate_summary(time_series: &[FlowMetricsResponse]) -> SummaryMetrics {
    let Some(latest) = time_series.last() else {
        return SummaryMetrics::default();
    };

    let previous = time_series.iter().rev().nth(1);

    let merge_rate = if latest.opened > 0 {
        ((latest.merged as f64 / latest.opened as f64) * 100.0).round() as u32
    } else {
        0
    };

    let is_widening = previous.is_some_and(|p| latest.spread > p.spread);

    SummaryMetrics {
        current_opened: latest.opened,
        current_merged: latest.merged,
        current_spread: latest.spread,
        merge_rate,
        is_widening,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github::PRState;
    use chrono::TimeZone;


    #[test]
    fn test_calculate_metrics_empty() {
        let now = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let days_to_display = Duration::days(1);
        let window_size = Duration::days(30);
        let response = calculate_metrics(&[], days_to_display, window_size, now);

        assert_eq!(response.time_series.len(), 2);
        assert_eq!(response.summary.current_opened, 0);
        assert_eq!(response.summary.merge_rate, 0);
    }

    #[test]
    fn test_calculate_metrics_with_data() {
        let now = Utc.with_ymd_and_hms(2024, 1, 10, 12, 0, 0).unwrap();

        let prs = vec![
            GitHubPR {
                id: 1,
                created_at: Utc.with_ymd_and_hms(2024, 1, 5, 10, 0, 0).unwrap(),
                merged_at: Some(Utc.with_ymd_and_hms(2024, 1, 6, 10, 0, 0).unwrap()),
                state: PRState::Merged,
            },
            GitHubPR {
                id: 2,
                created_at: Utc.with_ymd_and_hms(2024, 1, 9, 10, 0, 0).unwrap(),
                merged_at: None,
                state: PRState::Open,
            },
        ];

        let days_to_display = Duration::days(0); // Only today
        let window_size = Duration::days(30); // 30 day window
        let response = calculate_metrics(&prs, days_to_display, window_size, now);

        assert_eq!(response.time_series.len(), 1);
        assert_eq!(response.summary.current_opened, 2);
        assert_eq!(response.summary.current_merged, 1);
        assert_eq!(response.summary.merge_rate, 50);
    }

    #[test]
    fn test_calculate_summary_empty() {
        let metrics = calculate_summary(&[]);
        assert_eq!(metrics.current_opened, 0);
        assert_eq!(metrics.current_merged, 0);
        assert_eq!(metrics.current_spread, 0);
        assert_eq!(metrics.merge_rate, 0);
        assert_eq!(metrics.is_widening, false);
    }

}
