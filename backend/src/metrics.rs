use crate::github::GitHubPR;
use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeZone, Utc};
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
    let latest_date = now.date_naive();
    let oldest_display_date = latest_date - Duration::days(days_to_display_count);
    let start_date = oldest_display_date - Duration::days(window_size_days);

    // Initialize timelines for opened and merged events.
    let mut opened_timeline = Timeline::new(start_date, latest_date);
    let mut merged_timeline = Timeline::new(start_date, latest_date);

    // Populate timelines in a single pass.
    for pr in prs {
        opened_timeline.increment(pr.created_at.date_naive());
        if let Some(merged_at) = pr.merged_at {
            merged_timeline.increment(merged_at.date_naive());
        }
    }

    // Convert to prefix sums for O(1) range queries.
    let opened_prefix = opened_timeline.into_prefix_sums();
    let merged_prefix = merged_timeline.into_prefix_sums();

    // Generate the result time series.
    let time_series: Vec<FlowMetricsResponse> = (0..=days_to_display_count)
        .rev()
        .map(|i| {
            let date = now - Duration::days(i);
            let date_naive = date.date_naive();

            // Calculate rolling window sums using the prefix timelines.
            let opened = opened_prefix.sum_in_window(date_naive, window_size_days);
            let merged = merged_prefix.sum_in_window(date_naive, window_size_days);

            // We set the time to the end of the day to ensure we capture all activity for that date.
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

/// Helper struct to manage daily event counts over a date range.
struct Timeline {
    start_date: NaiveDate,
    end_date: NaiveDate,
    counts: Vec<usize>,
}

impl Timeline {
    fn new(start_date: NaiveDate, end_date: NaiveDate) -> Self {
        let days = (end_date - start_date).num_days();
        let size = if days >= 0 { (days + 1) as usize } else { 0 };
        Self {
            start_date,
            end_date,
            counts: vec![0; size],
        }
    }

    fn increment(&mut self, date: NaiveDate) {
        if date >= self.start_date && date <= self.end_date {
            let idx = (date - self.start_date).num_days() as usize;
            if idx < self.counts.len() {
                self.counts[idx] += 1;
            }
        }
    }

    fn into_prefix_sums(self) -> PrefixTimeline {
        let mut prefix_counts = Vec::with_capacity(self.counts.len());
        let mut sum = 0;
        for count in self.counts {
            sum += count;
            prefix_counts.push(sum);
        }
        PrefixTimeline {
            start_date: self.start_date,
            prefix_counts,
        }
    }
}

/// Helper struct to perform O(1) range sum queries using prefix sums.
struct PrefixTimeline {
    start_date: NaiveDate,
    prefix_counts: Vec<usize>,
}

impl PrefixTimeline {
    /// Returns the sum of events in the window (end_date - window_size, end_date].
    fn sum_in_window(&self, end_date: NaiveDate, window_size_days: i64) -> usize {
        let end_idx_signed = (end_date - self.start_date).num_days();

        // If the query date is before our start date, we have no data.
        if end_idx_signed < 0 {
            return 0;
        }

        let end_idx = end_idx_signed as usize;
        // Ensure we don't go out of bounds (though valid logic shouldn't).
        let end_val = if end_idx < self.prefix_counts.len() {
            self.prefix_counts[end_idx]
        } else {
            *self.prefix_counts.last().unwrap_or(&0)
        };

        // window start index = end_idx - window_size
        let start_idx_signed = end_idx_signed - window_size_days;

        let start_val = if start_idx_signed < 0 {
            0
        } else {
            let idx = start_idx_signed as usize;
             if idx < self.prefix_counts.len() {
                self.prefix_counts[idx]
            } else {
                *self.prefix_counts.last().unwrap_or(&0)
            }
        };

        end_val.saturating_sub(start_val)
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
