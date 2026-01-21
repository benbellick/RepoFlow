import type { RepoMetricsResponse } from '../types';

/**
 * Fetches repository metrics from the Rust backend API.
 *
 * @param owner - The GitHub username or organization.
 * @param repo - The repository name.
 * @returns A promise that resolves to a RepoMetricsResponse object.
 */
export const fetchRepoMetrics = async (owner: string, repo: string): Promise<RepoMetricsResponse> => {
  const response = await fetch(`/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/metrics`);

  if (!response.ok) {
    if (response.status === 404) {
      throw new Error('Repository not found or no metrics available.');
    }
    const errorText = await response.text();
    throw new Error(errorText || `Failed to fetch metrics: ${response.statusText}`);
  }

  return response.json();
};
