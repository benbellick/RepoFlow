use anyhow::Result;
use chrono::{DateTime, Utc};
use octocrab::Octocrab;
use serde::{Deserialize, Serialize};

/// Represents the state of a Pull Request.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PRState {
    Open,
    Closed,
    Merged,
    Unknown,
}

/// A simplified representation of a GitHub Pull Request for flow analysis.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GitHubPR {
    /// Unique identifier for the PR.
    pub id: u64,
    /// The timestamp when the PR was created.
    pub created_at: DateTime<Utc>,
    /// The timestamp when the PR was merged, if applicable.
    pub merged_at: Option<DateTime<Utc>>,
    /// The current state of the PR (open, closed, merged).
    pub state: PRState,
}

/// A client for interacting with the GitHub API using Octocrab.
pub struct GitHubClient {
    octocrab: Octocrab,
}

impl GitHubClient {
    /// Creates a new GitHubClient.
    ///
    /// # Arguments
    /// * `token` - An optional Personal Access Token for higher rate limits.
    pub fn new(token: Option<String>) -> Result<Self> {
        let mut builder = Octocrab::builder();
        if let Some(token) = token {
            builder = builder.personal_token(token);
        }

        Ok(Self {
            octocrab: builder.build()?,
        })
    }

    /// Fetches pull requests from a repository, traversing back a certain number of days.
    ///
    /// # Arguments
    /// * `owner` - The owner of the repository.
    /// * `repo` - The name of the repository.
    /// * `days` - How many days of history to fetch.
    /// * `max_pages` - The maximum number of API pages to fetch.
    pub async fn fetch_pull_requests(
        &self,
        owner: &str,
        repo: &str,
        days: i64,
        max_pages: u32,
    ) -> Result<Vec<GitHubPR>> {
        // Validation: Reject path traversal attempts or suspicious names.
        let owner = owner.trim();
        let repo = repo.trim();
        if owner.contains("..") || repo.contains("..") {
            return Err(anyhow::anyhow!(
                "Invalid owner or repo name: '..' is not allowed"
            ));
        }

        let mut prs = Vec::new();
        let cutoff_date = Utc::now() - chrono::Duration::days(days);

        let mut current_page = self
            .octocrab
            .pulls(owner, repo)
            .list()
            .state(octocrab::params::State::All)
            .sort(octocrab::params::pulls::Sort::Created)
            .direction(octocrab::params::Direction::Descending)
            .per_page(100)
            .send()
            .await?;

        let mut page_count = 1;

        loop {
            let mut reached_cutoff = false;

            for pr in &current_page {
                // Safely extract created_at, skipping malformed PRs.
                let created_at = match pr.created_at {
                    Some(dt) => dt,
                    None => continue,
                };

                if created_at >= cutoff_date {
                    // Determine state: Octocrab's `merged_at` is the source of truth for "merged".
                    let state = if pr.merged_at.is_some() {
                        PRState::Merged
                    } else {
                        match pr.state {
                            Some(octocrab::models::IssueState::Open) => PRState::Open,
                            Some(octocrab::models::IssueState::Closed) => PRState::Closed,
                            _ => PRState::Unknown,
                        }
                    };

                    prs.push(GitHubPR {
                        id: pr.id.into_inner(),
                        created_at,
                        merged_at: pr.merged_at,
                        state,
                    });
                } else {
                    reached_cutoff = true;
                    break;
                }
            }

            if reached_cutoff || page_count >= max_pages {
                break;
            }

            match self.octocrab.get_page(&current_page.next).await? {
                Some(next_page) => {
                    current_page = next_page;
                    page_count += 1;
                }
                None => break,
            }
        }

        Ok(prs)
    }
}
