use crate::config::AppConfig;
use crate::github::{GitHubClient, RepoId};
use crate::metrics::RepoMetricsResponse;
use moka::future::Cache;

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

        Self {
            cache,
            client,
            config: config.clone(),
        }
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
}



