use serde::Serialize;
use std::env;
use std::time::Duration as StdDuration;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub pr_fetch_days: i64,
    pub max_github_api_pages: u32,
    pub metrics_days_to_display: i64,
    pub metrics_window_size: i64,
    pub cache_ttl: StdDuration,
    pub cache_max_capacity: u64,
    pub max_concurrent_preloads: usize,
    pub popular_repos: Vec<PopularRepo>,
}

#[derive(Serialize, Clone, Debug)]
pub struct PopularRepo {
    pub owner: String,
    pub repo: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let pr_fetch_days = env::var("PR_FETCH_DAYS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(90);

        let max_github_api_pages = env::var("MAX_GITHUB_API_PAGES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10);

        let metrics_days_to_display = env::var("METRICS_DAYS_TO_DISPLAY")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30);

        let metrics_window_size = env::var("METRICS_WINDOW_SIZE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30);

        let cache_ttl_secs = env::var("CACHE_TTL_SECONDS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(86400);

        let cache_max_capacity = env::var("CACHE_MAX_CAPACITY")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1000);

        let max_concurrent_preloads = env::var("MAX_CONCURRENT_PRELOADS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(4);

        let popular_repos = env::var("POPULAR_REPOS")
            .ok()
            .map(|v| parse_popular_repos(&v))
            .unwrap_or_else(default_popular_repos);

        Self {
            pr_fetch_days,
            max_github_api_pages,
            metrics_days_to_display,
            metrics_window_size,
            cache_ttl: StdDuration::from_secs(cache_ttl_secs),
            cache_max_capacity,
            max_concurrent_preloads,
            popular_repos,
        }
    }
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

fn default_popular_repos() -> Vec<PopularRepo> {
    vec![
        PopularRepo {
            owner: "facebook".to_string(),
            repo: "react".to_string(),
        },
        PopularRepo {
            owner: "rust-lang".to_string(),
            repo: "rust".to_string(),
        },
        PopularRepo {
            owner: "vercel".to_string(),
            repo: "next.js".to_string(),
        },
        PopularRepo {
            owner: "tailwindlabs".to_string(),
            repo: "tailwindcss".to_string(),
        },
        PopularRepo {
            owner: "microsoft".to_string(),
            repo: "vscode".to_string(),
        },
        PopularRepo {
            owner: "rust-lang".to_string(),
            repo: "rust-analyzer".to_string(),
        },
    ]
}
