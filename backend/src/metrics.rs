use crate::github::GitHubPR;
use chrono::{DateTime, Datelike, Duration, TimeZone, Utc};
use serde::Serialize;

const END_OF_DAY_HOUR: u32 = 23;
const END_OF_DAY_MIN: u32 = 59;
const END_OF_DAY_SEC: u32 = 59;

/// The public response structure for flow metrics.
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
) -> Vec<FlowMetricsResponse> {
    (0..=days_to_display.num_days())
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
        .collect()
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
        let metrics = calculate_metrics(&[], days_to_display, window_size, now);

        assert_eq!(metrics.len(), 2);
        assert_eq!(metrics[0].opened, 0);
        assert_eq!(metrics[0].merged, 0);
        assert_eq!(metrics[0].spread, 0);
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
        let metrics = calculate_metrics(&prs, days_to_display, window_size, now);

        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].date, "2024-01-10");
        assert_eq!(metrics[0].opened, 2);
        assert_eq!(metrics[0].merged, 1);
        assert_eq!(metrics[0].spread, 1);
    }
}
