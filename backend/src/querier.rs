//! Service layer for querying and caching repository metrics.
//!
//! This module implements `MetricsQuerier`, which acts as the main entry point for retrieving
//! repository metrics. It handles:
//! 1. Checking the in-memory cache for existing data.
//! 2. Fetching raw data from GitHub if the cache is empty.
//! 3. Calculating domain-specific metrics from the raw data.
//! 4. Proactively refreshing popular repositories in the background.

use crate::config::AppConfig;
use crate::github::{GitHubPR, PRState, RepoId};
use crate::metrics::{self, RepoMetricsResponse};
use chrono::{Duration, Utc};
use moka::future::Cache;
use octocrab::models::pulls::PullRequest;
use octocrab::{Octocrab, Page};
use std::time::Duration as StdDuration;

#[derive(Clone)]
pub struct MetricsQuerier {
    cache: Cache<RepoId, RepoMetricsResponse>,
    octocrab: Octocrab,
    config: AppConfig,
}

impl MetricsQuerier {
    /// Initializes a new MetricsQuerier.
    ///
    /// This sets up the Octocrab client, the in-memory cache, and starts the background
    /// refresh task for popular repositories.
    pub fn new(config: &AppConfig) -> anyhow::Result<Self> {
        let mut builder = Octocrab::builder();
        if let Some(token) = &config.github_token {
            builder = builder.personal_token(token.clone());
        }
        let octocrab = builder.build()?;

        let cache = Cache::builder()
            .max_capacity(config.cache_max_capacity)
            .time_to_live(config.cache_ttl())
            .build();

        let querier = Self {
            cache,
            octocrab,
            config: config.clone(),
        };

        querier.start_background_refresh();

        Ok(querier)
    }

    /// Retrieves metrics for a repository, fetching them if not cached (read-through).
    pub async fn get(&self, repo_id: RepoId) -> anyhow::Result<RepoMetricsResponse> {
        if let Some(metrics) = self.cache.get(&repo_id).await {
            return Ok(metrics);
        }

        let metrics = self
            .fetch_and_calculate_metrics(&repo_id)
            .await?;

        self.cache.insert(repo_id, metrics.clone()).await;

        Ok(metrics)
    }

    /// Starts a background task that periodically refreshes metrics for popular repositories.
    fn start_background_refresh(&self) {
        let querier = self.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            // Refresh popular repos at half their TTL to ensure they are always fresh/warm.
            let mut interval =
                tokio::time::interval(StdDuration::from_secs(config.cache_ttl_seconds / 2));

            loop {
                interval.tick().await;

                for repo_id in &config.popular_repos {
                    if let Ok(metrics) = querier.fetch_and_calculate_metrics(repo_id).await {
                        querier.cache.insert(repo_id.clone(), metrics).await;
                    }
                }
            }
        });
    }

    /// Fetches PRs from GitHub and calculates flow metrics.
    async fn fetch_and_calculate_metrics(
        &self,
        repo_id: &RepoId,
    ) -> anyhow::Result<RepoMetricsResponse> {
        let prs = self
            .fetch_pull_requests(repo_id, self.config.pr_fetch_days, self.config.max_github_api_pages)
            .await?;

        let metrics = metrics::calculate_metrics(
            &prs,
            Duration::days(self.config.metrics_days_to_display),
            Duration::days(self.config.metrics_window_size),
            Utc::now(),
        );

        Ok(metrics)
    }

    /// Retrieves a list of pull requests for a specific repository.
    async fn fetch_pull_requests(
        &self,
        repo_id: &RepoId,
        days: i64,
        max_pages: u32,
    ) -> anyhow::Result<Vec<GitHubPR>> {
        let cutoff_date = Utc::now() - chrono::Duration::days(days);
        let mut prs = Vec::new();

        let mut current_page = self
            .octocrab
            .pulls(&repo_id.owner, &repo_id.repo)
            .list()
            .state(octocrab::params::State::All)
            .sort(octocrab::params::pulls::Sort::Created)
            .direction(octocrab::params::Direction::Descending)
            .per_page(100)
            .send()
            .await?;

        for _ in 1..=max_pages {
            let page_prs = self.process_pr_page(&current_page);
            prs.extend(page_prs);

            // If the last PR we just added is older than the cutoff, we can stop.
            if prs.last().is_some_and(|pr| pr.created_at < cutoff_date) {
                break;
            }

            if let Some(next_page) = self.octocrab.get_page(&current_page.next).await? {
                current_page = next_page;
            } else {
                break;
            }
        }

        // Clean up: remove any PRs that were in the last page but beyond the cutoff.
        prs.retain(|pr| pr.created_at >= cutoff_date);

        Ok(prs)
    }

    /// Processes a single page of Pull Requests, converting them to our internal type.
    fn process_pr_page(&self, page: &Page<PullRequest>) -> Vec<GitHubPR> {
        page.items
            .iter()
            .filter_map(|pr| {
                let created_at = pr.created_at?;

                let state = if pr.merged_at.is_some() {
                    PRState::Merged
                } else {
                    match pr.state {
                        Some(octocrab::models::IssueState::Open) => PRState::Open,
                        Some(octocrab::models::IssueState::Closed) => PRState::Closed,
                        Some(_) => PRState::Unknown,
                        None => PRState::Unknown,
                    }
                };

                Some(GitHubPR {
                    id: pr.id.into_inner(),
                    created_at,
                    merged_at: pr.merged_at,
                    state,
                })
            })
            .collect()
    }
}