/**
 * Represents the owner and repo name extracted from a URL.
 */
export interface RepoDetails {
  owner: string;
  repo: string;
}

/**
 * Parses a GitHub repository URL to extract the owner and repository name.
 *
 * @param url - The full GitHub URL (e.g., "https://github.com/facebook/react").
 * @returns A RepoDetails object if valid, or null if invalid.
 */
export const parseGitHubUrl = (url: string): RepoDetails | null => {
  try {
    const cleanUrl = url.trim().replace(/\/$/, '');
    const urlObj = new URL(cleanUrl);

    if (urlObj.hostname !== 'github.com') return null;

    const parts = urlObj.pathname.split('/').filter(Boolean);
    if (parts.length < 2) return null;

    return {
      owner: parts[0],
      repo: parts[1],
    };
  } catch {
    return null;
  }
};
