use crate::config::AppConfig;
use crate::fetcher;
use crate::github::GitHubClient;
use crate::metrics::RepoMetricsResponse;
use moka::future::Cache;
use moka::notification::RemovalCause;
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CacheKey {
    pub owner: String,
    pub repo: String,
}

#[derive(Clone)]
pub struct MetricsCache {
    cache: Cache<CacheKey, RepoMetricsResponse>,
}

impl MetricsCache {
    pub fn new(config: &AppConfig, github_client: GitHubClient) -> Self {
        // Channel for queuing expired keys to be refreshed
        let (tx, mut rx) = mpsc::unbounded_channel::<CacheKey>();

        // Eviction listener to capture expired keys
        let eviction_listener =
            move |key: Arc<CacheKey>, _value: RepoMetricsResponse, cause: RemovalCause| {
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
            while let Some(key) = rx.recv().await {
                tracing::info!(owner = %key.owner, repo = %key.repo, "Refreshing expired cache entry");

                match fetcher::fetch_and_calculate_metrics(
                    &client_clone,
                    &config_clone,
                    &key.owner,
                    &key.repo,
                )
                .await
                {
                    Ok(metrics) => {
                        cache_clone.insert(key.clone(), metrics).await;
                        tracing::info!(owner = %key.owner, repo = %key.repo, "Successfully refreshed cache");
                    }
                    Err(e) => {
                        tracing::error!(owner = %key.owner, repo = %key.repo, "Failed to refresh cache: {}", e);
                        // We do not re-insert the key, so it drops out of the cache loop.
                    }
                }
            }
        });

        Self { cache }
    }

    pub async fn get(&self, owner: &str, repo: &str) -> Option<RepoMetricsResponse> {
        let key = CacheKey {
            owner: owner.to_string(),
            repo: repo.to_string(),
        };
        self.cache.get(&key).await
    }

    pub async fn insert(&self, owner: String, repo: String, metrics: RepoMetricsResponse) {
        let key = CacheKey { owner, repo };
        self.cache.insert(key, metrics).await;
    }
}
