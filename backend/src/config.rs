//! Application configuration and environment variable parsing.
//!
//! This module handles loading configuration settings from the environment (e.g., .env file).
//! It defines the `AppConfig` struct which governs behavior such as API rate limits,
//! cache TTLs, and the list of popular repositories to preload.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration as StdDuration;

/// A unique identifier for a GitHub repository.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RepoId {
    /// The owner of the repository (e.g., "facebook").
    pub owner: String,
    /// The name of the repository (e.g., "react").
    pub repo: String,
}

impl fmt::Display for RepoId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.owner, self.repo)
    }
}

/// Application configuration loaded from environment variables.
#[derive(Clone, Debug, Deserialize)]
pub struct AppConfig {
    /// Number of past days to fetch pull request data for from the GitHub API.
    pub pr_fetch_days: i64,

    /// Hard limit on the number of paginated requests to make to the GitHub API per repository.
    pub max_github_api_pages: u32,

    /// The number of individual data points (days) to return in the flow metrics response.
    pub metrics_days_to_display: i64,

    /// The size of the trailing window (in days) used to calculate the rolling counts.
    pub metrics_window_size: i64,

    /// Time to live for cached repository metrics in seconds.
    pub cache_ttl_seconds: u64,

    /// Maximum number of entries to keep in the metrics cache.
    pub cache_max_capacity: u64,

    /// List of popular repositories to preload.
    /// Expected format: comma-separated string of "owner/repo" pairs.
    /// Example: "facebook/react,rust-lang/rust"
    #[serde(deserialize_with = "deserialize_popular_repos")]
    pub popular_repos: Vec<RepoId>,

    /// Maximum number of concurrent requests for refreshing popular repositories.
    /// Defaults to 10 if not specified.
    #[serde(default = "default_concurrency_limit")]
    pub popular_repos_concurrency_limit: usize,

    /// Optional GitHub Personal Access Token for higher rate limits.
    pub github_token: Option<String>,
}

fn default_concurrency_limit() -> usize {
    10
}

impl AppConfig {
    pub fn from_env() -> Result<Self, envy::Error> {
        envy::from_env()
    }

    pub fn cache_ttl(&self) -> StdDuration {
        StdDuration::from_secs(self.cache_ttl_seconds)
    }
}

fn deserialize_popular_repos<'de, D>(deserializer: D) -> Result<Vec<RepoId>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(parse_popular_repos(&s))
}

fn parse_popular_repos(s: &str) -> Vec<RepoId> {
    s.split(',')
        .filter_map(|part| {
            let parts: Vec<&str> = part.trim().split('/').collect();
            if parts.len() == 2 {
                Some(RepoId {
                    owner: parts[0].trim().to_string(),
                    repo: parts[1].trim().to_string(),
                })
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;

    #[test]
    #[serial]
    fn test_config_from_env() {
        // Set env vars
        env::set_var("PR_FETCH_DAYS", "100");
        env::set_var("MAX_GITHUB_API_PAGES", "5");
        env::set_var("METRICS_DAYS_TO_DISPLAY", "15");
        env::set_var("METRICS_WINDOW_SIZE", "15");
        env::set_var("CACHE_TTL_SECONDS", "3600");
        env::set_var("CACHE_MAX_CAPACITY", "500");
        env::set_var("POPULAR_REPOS", "owner1/repo1,owner2/repo2");
        env::set_var("POPULAR_REPOS_CONCURRENCY_LIMIT", "5");

        let config = AppConfig::from_env().expect("Failed to load config");

        assert_eq!(config.pr_fetch_days, 100);
        assert_eq!(config.max_github_api_pages, 5);
        assert_eq!(config.metrics_days_to_display, 15);
        assert_eq!(config.metrics_window_size, 15);
        assert_eq!(config.cache_ttl_seconds, 3600);
        assert_eq!(config.cache_max_capacity, 500);
        assert_eq!(config.popular_repos.len(), 2);
        assert_eq!(config.popular_repos[0].owner, "owner1");
        assert_eq!(config.popular_repos[0].repo, "repo1");
        assert_eq!(config.popular_repos_concurrency_limit, 5);

        // Clean up
        env::remove_var("PR_FETCH_DAYS");
        env::remove_var("MAX_GITHUB_API_PAGES");
        env::remove_var("METRICS_DAYS_TO_DISPLAY");
        env::remove_var("METRICS_WINDOW_SIZE");
        env::remove_var("CACHE_TTL_SECONDS");
        env::remove_var("CACHE_MAX_CAPACITY");
        env::remove_var("POPULAR_REPOS");
        env::remove_var("POPULAR_REPOS_CONCURRENCY_LIMIT");
    }

    #[test]
    #[serial]
    fn test_config_missing_vars() {
        // Ensure a var is missing
        env::remove_var("PR_FETCH_DAYS");
        let result = AppConfig::from_env();
        assert!(result.is_err());
    }
}
