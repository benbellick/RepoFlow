use crate::{config::AppConfig, github::GitHubClient, metrics};
use chrono::{Duration, Utc};
use moka::{future::Cache, notification::RemovalCause};
use std::sync::Arc;
use tokio::sync::mpsc;

/// A structured key for the metrics cache.
#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct CacheKey {
    pub owner: String,
    pub repo: String,
}

/// A wrapper around the metrics cache that handles automatic background refreshing of expired entries.
#[derive(Clone)]
pub struct MetricsCache {
    inner: Cache<CacheKey, metrics::RepoMetricsResponse>,
    config: AppConfig,
}

impl MetricsCache {
    /// Creates a new `MetricsCache` and spawns a background task to refresh expired entries.
    ///
    /// # Arguments
    /// * `github_client` - The client used to fetch data from GitHub during refresh.
    /// * `config` - The application configuration.
    pub fn new(github_client: GitHubClient, config: &AppConfig) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<Arc<CacheKey>>();

        let cache = Cache::builder()
            .max_capacity(config.cache_max_capacity)
            .time_to_live(config.cache_ttl())
            .eviction_listener(move |key: Arc<CacheKey>, _value, cause| {
                if cause == RemovalCause::Expired {
                    tracing::info!(
                        "Cache entry expired for key: {:?}/{:?}",
                        key.owner,
                        key.repo
                    );
                    if let Err(e) = tx.send(key) {
                        tracing::error!("Failed to send expired key to refresh channel: {}", e);
                    }
                }
            })
            .build();

        let metrics_cache = Self {
            inner: cache,
            config: config.clone(),
        };
        let refresher = metrics_cache.clone();

        // Spawn background refresh task
        tokio::spawn(async move {
            while let Some(key) = rx.recv().await {
                tracing::info!("Refreshing expired metrics for {}/{}", key.owner, key.repo);
                match refresher
                    .fetch_and_calculate_metrics(&github_client, &key.owner, &key.repo)
                    .await
                {
                    Ok(metrics) => {
                        refresher.inner.insert(key.as_ref().clone(), metrics).await;
                        tracing::info!(
                            "Successfully refreshed metrics for {}/{}",
                            key.owner,
                            key.repo
                        );
                    }
                    Err((status, msg)) => {
                        tracing::error!(
                            "Failed to refresh metrics for {}/{}: {} - {}",
                            key.owner,
                            key.repo,
                            status,
                            msg
                        );
                    }
                }
            }
        });

        metrics_cache
    }

    /// Retrieves metrics from the cache or fetches them if missing.
    ///
    /// # Arguments
    /// * `github_client` - The client to use for fetching if data is missing.
    /// * `owner` - The repository owner.
    /// * `repo` - The repository name.
    pub async fn get(
        &self,
        github_client: &GitHubClient,
        owner: &str,
        repo: &str,
    ) -> Result<metrics::RepoMetricsResponse, (axum::http::StatusCode, String)> {
        let key = CacheKey {
            owner: owner.to_string(),
            repo: repo.to_string(),
        };

        if let Some(metrics) = self.inner.get(&key).await {
            tracing::debug!(owner = %owner, repo = %repo, "Returning cached metrics");
            return Ok(metrics);
        }

        let metrics = self
            .fetch_and_calculate_metrics(github_client, owner, repo)
            .await?;
        self.inner.insert(key, metrics.clone()).await;

        Ok(metrics)
    }

    /// Fetches PR data from GitHub and calculates flow metrics.
    async fn fetch_and_calculate_metrics(
        &self,
        github_client: &GitHubClient,
        owner: &str,
        repo: &str,
    ) -> Result<metrics::RepoMetricsResponse, (axum::http::StatusCode, String)> {
        let prs = github_client
            .fetch_pull_requests(
                owner,
                repo,
                self.config.pr_fetch_days,
                self.config.max_github_api_pages,
            )
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch PRs for {}/{}: {}", owner, repo, e);

                if let Some(octocrab::Error::GitHub { source, .. }) =
                    e.downcast_ref::<octocrab::Error>()
                {
                    if source.message.to_lowercase().contains("rate limit") {
                        return (
                            axum::http::StatusCode::TOO_MANY_REQUESTS,
                            "GitHub Rate Limit Exceeded".to_string(),
                        );
                    }
                    if source.message.to_lowercase().contains("not found") {
                        return (
                            axum::http::StatusCode::NOT_FOUND,
                            "Repository Not Found".to_string(),
                        );
                    }
                }

                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error".to_string(),
                )
            })?;

        let metrics = metrics::calculate_metrics(
            &prs,
            Duration::days(self.config.metrics_days_to_display),
            Duration::days(self.config.metrics_window_size),
            Utc::now(),
        );

        Ok(metrics)
    }
}
