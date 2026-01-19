use crate::github::GitHubPR;
use chrono::{Datelike, TimeZone, Utc};
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
    let mut data = Vec::new();
    let now = Utc::now();

    for i in (0..=days_to_display).rev() {
        let target_date = now - chrono::Duration::days(i);
        // Set to end of day
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
                if let Some(merged_at) = pr.merged_at {
                    merged_at >= window_start && merged_at <= target_date
                } else {
                    false
                }
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
