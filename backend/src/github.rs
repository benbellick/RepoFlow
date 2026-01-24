//! GitHub data models.
//!
//! This module defines domain-specific types like `RepoId` and `GitHubPR`
//! that abstract away the complexity of the raw GitHub API responses.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

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
