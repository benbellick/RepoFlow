use anyhow::Result;
use chrono::{DateTime, Utc};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GitHubPR {
    pub id: u64,
    pub created_at: DateTime<Utc>,
    pub merged_at: Option<DateTime<Utc>>,
    pub state: String,
}

pub struct GitHubClient {
    client: reqwest::Client,
    token: Option<String>,
}

impl GitHubClient {
    pub fn new(token: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            token,
        }
    }

    pub async fn fetch_pull_requests(
        &self,
        owner: &str,
        repo: &str,
        days: i64,
    ) -> Result<Vec<GitHubPR>> {
        let mut prs = Vec::new();
        let mut page = 1;
        let per_page = 100;
        let cutoff_date = Utc::now() - chrono::Duration::days(days);

        loop {
            let url = format!(
                "https://api.github.com/repos/{}/{}/pulls?state=all&sort=created&direction=desc&per_page={}&page={}",
                owner, repo, per_page, page
            );

            let mut headers = HeaderMap::new();
            headers.insert(USER_AGENT, HeaderValue::from_static("repoflow-backend"));
            headers.insert(
                "Accept",
                HeaderValue::from_static("application/vnd.github.v3+json"),
            );

            if let Some(ref token) = self.token {
                headers.insert(
                    AUTHORIZATION,
                    HeaderValue::from_str(&format!("token {}", token))?,
                );
            }

            let response = self.client.get(&url).headers(headers).send().await?;

            if !response.status().is_success() {
                if response.status() == 403 {
                    return Err(anyhow::anyhow!("GitHub API rate limit exceeded"));
                }
                return Err(anyhow::anyhow!(
                    "Failed to fetch PRs: {}",
                    response.status()
                ));
            }

            let data: Vec<GitHubPR> = response.json().await?;
            if data.is_empty() {
                break;
            }

            let mut reached_cutoff = false;
            for pr in data {
                if pr.created_at < cutoff_date {
                    reached_cutoff = true;
                }
                prs.push(pr);
            }

            if reached_cutoff || page >= 10 {
                break;
            }
            page += 1;
        }

        Ok(prs)
    }
}
