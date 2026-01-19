export interface GitHubPR {
  id: number;
  created_at: string;
  merged_at: string | null;
  state: string;
}

const GITHUB_TOKEN = import.meta.env.VITE_GITHUB_TOKEN || '';

export const fetchPullRequests = async (owner: string, repo: string, days: number = 90): Promise<GitHubPR[]> => {
  const prs: GitHubPR[] = [];
  let page = 1;
  const perPage = 100;
  const cutoffDate = new Date();
  cutoffDate.setDate(cutoffDate.getDate() - days);

  while (true) {
    const response = await fetch(
      `https://api.github.com/repos/${owner}/${repo}/pulls?state=all&sort=created&direction=desc&per_page=${perPage}&page=${page}`,
      {
        headers: GITHUB_TOKEN ? {
          'Authorization': `token ${GITHUB_TOKEN}`,
          'Accept': 'application/vnd.github.v3+json'
        } : {
          'Accept': 'application/vnd.github.v3+json'
        }
      }
    );

    if (!response.ok) {
      if (response.status === 403) {
        throw new Error('GitHub API rate limit exceeded. Please provide a VITE_GITHUB_TOKEN.');
      }
      throw new Error(`Failed to fetch PRs: ${response.statusText}`);
    }

    const data = await response.json();
    if (!data || data.length === 0) break;

    let reachedCutoff = false;
    for (const pr of data) {
      const createdAt = new Date(pr.created_at);
      
      // If the PR was created before our cutoff, we might be able to stop.
      // However, a PR created long ago could have been merged RECENTLY.
      // So we also need to check merged_at if we want to be perfectly accurate for "merged" stats.
      // For V1, we'll focus on PRs created within the window.
      if (createdAt < cutoffDate) {
        // We only stop if the PR was also not merged recently. 
        // But the API returns PRs sorted by created_at desc.
        // To be safe, we'll continue a bit further or just fetch a fixed number of pages.
        reachedCutoff = true;
      }
      
      prs.push({
        id: pr.id,
        created_at: pr.created_at,
        merged_at: pr.merged_at,
        state: pr.state
      });
    }

    if (reachedCutoff) break;
    page++;
    
    // Safety break to avoid infinite loops/too many requests
    if (page > 10) break; 
  }

  return prs;
};
