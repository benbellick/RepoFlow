use crate::github::GitHubPR;
use chrono::{DateTime, Datelike, TimeZone, Utc};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct FlowMetricsResponse {
    pub date: String,
    pub opened: usize,
    pub merged: usize,
    pub spread: i64,
}

pub fn calculate_metrics(
    prs: &[GitHubPR],
    days_to_display: i64,
    window_size: i64,
) -> Vec<FlowMetricsResponse> {
    calculate_metrics_at(prs, days_to_display, window_size, Utc::now())
}

fn calculate_metrics_at(
    prs: &[GitHubPR],
    days_to_display: i64,
    window_size: i64,
    now: DateTime<Utc>,
) -> Vec<FlowMetricsResponse> {
    let mut data = Vec::new();

    for i in (0..=days_to_display).rev() {
        let target_date = now - chrono::Duration::days(i);
        let target_date = Utc
            .with_ymd_and_hms(
                target_date.year(),
                target_date.month(),
                target_date.day(),
                23,
                59,
                59,
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
    use chrono::TimeZone;

    #[test]
    fn test_calculate_metrics_empty() {
        let now = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let metrics = calculate_metrics_at(&[], 1, 30, now);

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
                state: "merged".to_string(),
            },
            GitHubPR {
                id: 2,
                created_at: Utc.with_ymd_and_hms(2024, 1, 9, 10, 0, 0).unwrap(),
                merged_at: None,
                state: "open".to_string(),
            },
        ];

        // 30 day window ending on Jan 10
        let metrics = calculate_metrics_at(&prs, 0, 30, now);

        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].date, "2024-01-10");
        assert_eq!(metrics[0].opened, 2);
        assert_eq!(metrics[0].merged, 1);
        assert_eq!(metrics[0].spread, 1);
    }
}
