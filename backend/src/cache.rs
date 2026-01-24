//! InMemory caching layer for repository metrics.
//!
//! This module implements a read-through cache (`MetricsCache`) that stores calculated metrics
//! for requested repositories. It improves performance and reduces GitHub API usage by serving
//! cached data for repeat requests.
//!
//! In addition to standard caching, it features a background refresh mechanism that proactively
//! updates "popular" repositories (defined in config) to ensure they are always warm and fresh.

use crate::config::AppConfig;
use crate::github::{GitHubClient, RepoId};
use crate::metrics::RepoMetricsResponse;
use moka::future::Cache;
use std::time::Duration;

#[derive(Clone)]
pub struct MetricsCache {
    cache: Cache<RepoId, RepoMetricsResponse>,
    client: GitHubClient,
    config: AppConfig,
}

impl MetricsCache {
    pub fn new(config: &AppConfig, client: GitHubClient) -> Self {
        let cache = Cache::builder()
            .max_capacity(config.cache_max_capacity)
            .time_to_live(config.cache_ttl())
            .build();

        let metrics_cache = Self {
            cache,
            client,
            config: config.clone(),
        };

        metrics_cache.start_background_refresh();

        metrics_cache
    }

    /// Retrieves metrics for a repository, fetching them if not cached (read-through).
    pub async fn get(&self, repo_id: RepoId) -> anyhow::Result<RepoMetricsResponse> {
        if let Some(metrics) = self.cache.get(&repo_id).await {
            return Ok(metrics);
        }

        let metrics = self
            .client
            .fetch_and_calculate_metrics(&self.config, &repo_id)
            .await?;

        self.cache.insert(repo_id, metrics.clone()).await;

        Ok(metrics)
    }

    /// Starts a background task that periodically refreshes metrics for popular repositories.
    ///
    /// By updating the cache before entries expire, we ensure that popular repositories
    /// always serve fresh data without blocking user requests.
    fn start_background_refresh(&self) {
        let cache = self.cache.clone();
        let client = self.client.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            // Refresh popular repos at half their TTL to ensure they are always fresh/warm.
            let mut interval =
                tokio::time::interval(Duration::from_secs(config.cache_ttl_seconds / 2));

            loop {
                interval.tick().await;

                for repo_id in &config.popular_repos {
                    if let Ok(metrics) = client.fetch_and_calculate_metrics(&config, repo_id).await {
                        cache.insert(repo_id.clone(), metrics).await;
                    }
                }
            }
        });
    }
}