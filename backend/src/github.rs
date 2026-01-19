use anyhow::Result;
use chrono::{DateTime, Utc};
use octocrab::Octocrab;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GitHubPR {
    pub id: u64,
    pub created_at: DateTime<Utc>,
    pub merged_at: Option<DateTime<Utc>>,
    pub state: String,
}

pub struct GitHubClient {
    octocrab: Octocrab,
}

impl GitHubClient {
    pub fn new(token: Option<String>) -> Result<Self> {
        let mut builder = Octocrab::builder();
        if let Some(token) = token {
            builder = builder.personal_token(token);
        }

        Ok(Self {
            octocrab: builder.build()?,
        })
    }

    pub async fn fetch_pull_requests(
        &self,
        owner: &str,
        repo: &str,
        days: i64,
        max_pages: u32,
    ) -> Result<Vec<GitHubPR>> {
        // Sanitize inputs to prevent path traversal or unintended endpoint access
        let owner = owner.trim().replace("..", "");
        let repo = repo.trim().replace("..", "");

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
                let created_at = pr.created_at.expect("PR missing created_at");

                if created_at >= cutoff_date {
                    prs.push(GitHubPR {
                        id: pr.id.into_inner(),
                        created_at,
                        merged_at: pr.merged_at,
                        state: format!("{:?}", pr.state),
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
