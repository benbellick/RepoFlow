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
#[derive(Debug, Serialize, Clone)]
pub struct SummaryMetrics {
    pub current_opened: usize,
    pub current_merged: usize,
    pub current_spread: i64,
    pub merge_rate: u32,
    pub is_widening: bool,
}

/// A single data point in the flow metrics time series.
#[derive(Debug, Serialize, Clone)]
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
    let time_series: Vec<FlowMetricsResponse> = (0..=days_to_display.num_days())
        .rev()
        .map(|i| {
            let date = now - Duration::days(i);
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

            calculate_day_metrics(prs, target_date, window_size)
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
    let latest = time_series.last().cloned().unwrap_or(FlowMetricsResponse {
        date: "".to_string(),
        opened: 0,
        merged: 0,
        spread: 0,
    });

    let previous = if time_series.len() > 1 {
        time_series.get(time_series.len() - 2)
    } else {
        None
    };

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

/// Calculates opened and merged metrics for a single point in time using a rolling window.
fn calculate_day_metrics(
    prs: &[GitHubPR],
    target_date: DateTime<Utc>,
    window_size: Duration,
) -> FlowMetricsResponse {
    let window_start = target_date - window_size;

    let opened = prs
        .iter()
        .filter(|pr| pr.created_at >= window_start && pr.created_at <= target_date)
        .count();

    let merged = prs
        .iter()
        .filter(|pr| {
            pr.merged_at
                .is_some_and(|merged_at| merged_at >= window_start && merged_at <= target_date)
        })
        .count();

    FlowMetricsResponse {
        date: target_date.format("%Y-%m-%d").to_string(),
        opened,
        merged,
        spread: opened as i64 - merged as i64,
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
}
