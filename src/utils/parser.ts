export interface RepoDetails {
  owner: string;
  repo: string;
}

export const parseGitHubUrl = (url: string): RepoDetails | null => {
  try {
    const cleanUrl = url.trim().replace(/\/$/, '');
    const urlObj = new URL(cleanUrl);
    
    if (urlObj.hostname !== 'github.com') return null;
    
    const parts = urlObj.pathname.split('/').filter(Boolean);
    if (parts.length < 2) return null;
    
    return {
      owner: parts[0],
      repo: parts[1]
    };
  } catch {
    return null;
  }
};
