use crate::config::AppConfig;
use crate::github::GitHubClient;
use crate::metrics::{self, RepoMetricsResponse};
use anyhow::Result;
use chrono::{Duration, Utc};

/// Fetches PRs from GitHub and calculates flow metrics.
///
/// This function separates the logic of data retrieval and processing
/// from the HTTP layer.
pub async fn fetch_and_calculate_metrics(
    client: &GitHubClient,
    config: &AppConfig,
    owner: &str,
    repo: &str,
) -> Result<RepoMetricsResponse> {
    let prs = client
        .fetch_pull_requests(
            owner,
            repo,
            config.pr_fetch_days,
            config.max_github_api_pages,
        )
        .await?;

    let metrics = metrics::calculate_metrics(
        &prs,
        Duration::days(config.metrics_days_to_display),
        Duration::days(config.metrics_window_size),
        Utc::now(),
    );

    Ok(metrics)
}
