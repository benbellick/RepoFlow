use anyhow::Result;
use chrono::{DateTime, Utc};
use octocrab::Octocrab;
use serde::{Deserialize, Serialize};

/// Represents the possible states of a GitHub Pull Request in our system.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PRState {
    /// The PR is currently open and active.
    Open,
    /// The PR has been closed without being merged.
    Closed,
    /// The PR has been successfully merged into the target branch.
    Merged,
    /// The state of the PR could not be determined.
    Unknown,
}

/// A simplified representation of a GitHub Pull Request used for calculating flow metrics.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GitHubPR {
    /// The unique GitHub database ID for this pull request.
    pub id: u64,
    /// The exact timestamp when the pull request was first opened.
    pub created_at: DateTime<Utc>,
    /// The timestamp when the pull request was merged (None if not merged).
    pub merged_at: Option<DateTime<Utc>>,
    /// The current operational state of the pull request.
    pub state: PRState,
}

/// A client wrapper around `octocrab::Octocrab` for fetching repository data.
#[derive(Clone)]
pub struct GitHubClient {
    /// The underlying Octocrab instance used for API requests.
    octocrab: Octocrab,
}

impl GitHubClient {
    /// Initializes a new GitHubClient with an optional authentication token.
    ///
    /// # Arguments
    /// * `token` - A GitHub Personal Access Token (PAT). Recommended to avoid rate limits.
    pub fn new(token: Option<String>) -> Result<Self> {
        let mut builder = Octocrab::builder();
        if let Some(token) = token {
            builder = builder.personal_token(token);
        }

        Ok(Self {
            octocrab: builder.build()?,
        })
    }

    /// Retrieves a list of pull requests for a specific repository.
    ///
    /// This method handles pagination automatically and filters PRs based on a date cutoff.
    /// It stops fetching as soon as it encounters a PR older than the specified `days`.
    ///
    /// # Arguments
    /// * `owner` - The GitHub username or organization (e.g., "facebook").
    /// * `repo` - The repository name (e.g., "react").
    /// * `days` - The number of days of history to fetch from the current time.
    /// * `max_pages` - The maximum number of API pages to traverse (safety limit).
    pub async fn fetch_pull_requests(
        &self,
        owner: &str,
        repo: &str,
        days: i64,
        max_pages: u32,
    ) -> Result<Vec<GitHubPR>> {
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
                let created_at = match pr.created_at {
                    Some(dt) => dt,
                    None => continue,
                };

                if created_at >= cutoff_date {
                    let state = if pr.merged_at.is_some() {
                        PRState::Merged
                    } else {
                        match pr.state {
                            Some(octocrab::models::IssueState::Open) => PRState::Open,
                            Some(octocrab::models::IssueState::Closed) => PRState::Closed,
                            Some(_) => PRState::Unknown,
                            None => PRState::Unknown,
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
