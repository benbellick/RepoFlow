import type { RepoMetricsResponse, PopularRepo } from '../types'

/**
 * Fetches repository metrics from the Rust backend API.
 *
 * @param owner - The GitHub username or organization.
 * @param repo - The repository name.
 * @param signal - Optional AbortSignal to cancel the request.
 * @returns A promise that resolves to a RepoMetricsResponse object.
 */
export const fetchRepoMetrics = async (
  owner: string,
  repo: string,
  signal?: AbortSignal,
): Promise<RepoMetricsResponse> => {
  const response = await fetch(
    `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/metrics`,
    { signal },
  )

  if (!response.ok) {
    if (response.status === 404) {
      throw new Error('Repository not found or no metrics available.')
    }
    const errorText = await response.text()
    throw new Error(errorText || `Failed to fetch metrics: ${response.statusText}`)
  }

  return response.json()
}
/**
 * Fetches the list of popular repositories from the backend.
 *
 * @returns A promise that resolves to an array of PopularRepo objects.
 */
export const fetchPopularRepos = async (): Promise<PopularRepo[]> => {
  const response = await fetch('/api/repos/popular')

  if (!response.ok) {
    throw new Error(`Failed to fetch popular repos: ${response.statusText}`)
  }

  return response.json()
}
