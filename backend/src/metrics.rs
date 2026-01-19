use crate::github::GitHubPR;
use chrono::{DateTime, Datelike, TimeZone, Utc};
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
/// * `window_size` - The size of the rolling window in days (e.g., 30 for a 30-day average).
pub fn calculate_metrics(
    prs: &[GitHubPR],
    days_to_display: i64,
    window_size: i64,
) -> Vec<FlowMetricsResponse> {
    calculate_metrics_at(prs, days_to_display, window_size, Utc::now())
}

/// Internal helper to allow deterministic testing of metrics calculation.
fn calculate_metrics_at(
    prs: &[GitHubPR],
    days_to_display: i64,
    window_size: i64,
    now: DateTime<Utc>,
) -> Vec<FlowMetricsResponse> {
    let mut data = Vec::new();

    for i in (0..=days_to_display).rev() {
        let target_date = now - chrono::Duration::days(i);
        // We set the time to the end of the day to ensure we capture all activity for that date.
        let target_date = Utc
            .with_ymd_and_hms(
                target_date.year(),
                target_date.month(),
                target_date.day(),
                END_OF_DAY_HOUR,
                END_OF_DAY_MIN,
                END_OF_DAY_SEC,
            )
            .unwrap();

        let window_start = target_date - chrono::Duration::days(window_size);

        let opened_in_window = prs
            .iter()
            .filter(|pr| pr.created_at >= window_start && pr.created_at <= target_date)
            .count();

        let merged_in_window = prs
            .iter()
            .filter(|pr| {
                pr.merged_at
                    .is_some_and(|merged_at| merged_at >= window_start && merged_at <= target_date)
            })
            .count();

        data.push(FlowMetricsResponse {
            date: target_date.format("%Y-%m-%d").to_string(),
            opened: opened_in_window,
            merged: merged_in_window,
            spread: opened_in_window as i64 - merged_in_window as i64,
        });
    }

    data
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github::PRState;
    use chrono::TimeZone;

    #[test]
    fn test_calculate_metrics_empty() {
        let now = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let days_to_display = 1;
        let window_size = 30;
        let metrics = calculate_metrics_at(&[], days_to_display, window_size, now);

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

        let days_to_display = 0; // Only today
        let window_size = 30; // 30 day window
        let metrics = calculate_metrics_at(&prs, days_to_display, window_size, now);

        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].date, "2024-01-10");
        assert_eq!(metrics[0].opened, 2);
        assert_eq!(metrics[0].merged, 1);
        assert_eq!(metrics[0].spread, 1);
    }
}
