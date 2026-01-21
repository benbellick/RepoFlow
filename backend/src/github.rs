use chrono::{DateTime, Utc};
use octocrab::models::pulls::PullRequest;
use octocrab::{Octocrab, Page};
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
    #[allow(clippy::result_large_err)]
    pub fn new(token: Option<String>) -> octocrab::Result<Self> {
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
    ) -> octocrab::Result<Vec<GitHubPR>> {
        let cutoff_date = Utc::now() - chrono::Duration::days(days);
        let mut prs = Vec::new();

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

        for _ in 1..=max_pages {
            let page_prs = self.process_pr_page(&current_page);
            prs.extend(page_prs);

            // If the last PR we just added is older than the cutoff, we can stop.
            if prs.last().is_some_and(|pr| pr.created_at < cutoff_date) {
                break;
            }

            if let Some(next_page) = self.octocrab.get_page(&current_page.next).await? {
                current_page = next_page;
            } else {
                break;
            }
        }

        // Clean up: remove any PRs that were in the last page but beyond the cutoff.
        prs.retain(|pr| pr.created_at >= cutoff_date);

        Ok(prs)
    }

    /// Processes a single page of Pull Requests, converting them to our internal type.
    fn process_pr_page(&self, page: &Page<PullRequest>) -> Vec<GitHubPR> {
        page.items
            .iter()
            .filter_map(|pr| {
                let created_at = pr.created_at?;

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

                Some(GitHubPR {
                    id: pr.id.into_inner(),
                    created_at,
                    merged_at: pr.merged_at,
                    state,
                })
            })
            .collect()
    }
}
