use serde::Deserialize;
use serde::Serialize;
use std::time::Duration as StdDuration;

#[derive(Clone, Debug, Deserialize)]
pub struct AppConfig {
    pub pr_fetch_days: i64,

    pub max_github_api_pages: u32,

    pub metrics_days_to_display: i64,

    pub metrics_window_size: i64,

    pub cache_ttl_seconds: u64,

    pub cache_max_capacity: u64,

    pub max_concurrent_preloads: usize,

    #[serde(deserialize_with = "deserialize_popular_repos")]
    pub popular_repos: Vec<PopularRepo>,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, envy::Error> {
        envy::from_env()
    }

    pub fn cache_ttl(&self) -> StdDuration {
        StdDuration::from_secs(self.cache_ttl_seconds)
    }
}

#[derive(Serialize, Clone, Debug, Deserialize, PartialEq)]
pub struct PopularRepo {
    pub owner: String,
    pub repo: String,
}

fn deserialize_popular_repos<'de, D>(deserializer: D) -> Result<Vec<PopularRepo>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(parse_popular_repos(&s))
}

fn parse_popular_repos(s: &str) -> Vec<PopularRepo> {
    s.split(',')
        .filter_map(|part| {
            let parts: Vec<&str> = part.trim().split('/').collect();
            if parts.len() == 2 {
                Some(PopularRepo {
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
        env::set_var("MAX_CONCURRENT_PRELOADS", "2");
        env::set_var("POPULAR_REPOS", "owner1/repo1,owner2/repo2");

        let config = AppConfig::from_env().expect("Failed to load config");

        assert_eq!(config.pr_fetch_days, 100);
        assert_eq!(config.max_github_api_pages, 5);
        assert_eq!(config.metrics_days_to_display, 15);
        assert_eq!(config.metrics_window_size, 15);
        assert_eq!(config.cache_ttl_seconds, 3600);
        assert_eq!(config.cache_max_capacity, 500);
        assert_eq!(config.max_concurrent_preloads, 2);
        assert_eq!(config.popular_repos.len(), 2);
        assert_eq!(config.popular_repos[0].owner, "owner1");
        assert_eq!(config.popular_repos[0].repo, "repo1");

        // Clean up
        env::remove_var("PR_FETCH_DAYS");
        env::remove_var("MAX_GITHUB_API_PAGES");
        env::remove_var("METRICS_DAYS_TO_DISPLAY");
        env::remove_var("METRICS_WINDOW_SIZE");
        env::remove_var("CACHE_TTL_SECONDS");
        env::remove_var("CACHE_MAX_CAPACITY");
        env::remove_var("MAX_CONCURRENT_PRELOADS");
        env::remove_var("POPULAR_REPOS");
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
