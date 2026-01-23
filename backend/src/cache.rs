use crate::config::AppConfig;
use crate::github::GitHubClient;
use crate::metrics::RepoMetricsResponse;
use crate::github::RepoId;
use moka::future::Cache;
use moka::notification::RemovalCause;
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct MetricsCache {
    cache: Cache<RepoId, RepoMetricsResponse>,
}

impl MetricsCache {
    pub fn new(config: &AppConfig, github_client: GitHubClient) -> Self {
        // Channel for queuing expired keys to be refreshed
        let (tx, mut rx) = mpsc::unbounded_channel::<RepoId>();

        // Eviction listener to capture expired keys
        let eviction_listener = move |key: Arc<RepoId>, _value: RepoMetricsResponse, cause: RemovalCause| {
            if cause == RemovalCause::Expired {
                // Send the key to the refresh worker
                // We clone the key because the listener receives an Arc<K>
                if let Err(e) = tx.send((*key).clone()) {
                    tracing::error!("Failed to queue expired key for refresh: {}", e);
                }
            }
        };

        let cache = Cache::builder()
            .max_capacity(config.cache_max_capacity)
            .time_to_live(config.cache_ttl())
            .eviction_listener(eviction_listener)
            .build();

        let cache_clone = cache.clone();
        let config_clone = config.clone();
        let client_clone = github_client.clone();

        // Background worker to refresh expired keys
        tokio::spawn(async move {
            while let Some(repo_id) = rx.recv().await {
                tracing::info!(repo_id = %repo_id, "Refreshing expired cache entry");

                match client_clone.fetch_and_calculate_metrics(&config_clone, &repo_id.owner, &repo_id.repo).await {
                    Ok(metrics) => {
                        cache_clone.insert(repo_id.clone(), metrics).await;
                        tracing::info!(repo_id = %repo_id, "Successfully refreshed cache");
                    }
                    Err(e) => {
                        tracing::error!(repo_id = %repo_id, "Failed to refresh cache: {}", e);
                        // We do not re-insert the key, so it drops out of the cache loop.
                    }
                }
            }
        });

        Self { cache }
    }

    pub async fn get(&self, repo_id: &RepoId) -> Option<RepoMetricsResponse> {
        self.cache.get(repo_id).await
    }

    pub async fn insert(&self, repo_id: RepoId, metrics: RepoMetricsResponse) {
        self.cache.insert(repo_id, metrics).await;
    }
}
