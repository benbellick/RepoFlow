import { describe, it, expect } from 'vitest';
import { parseGitHubUrl } from './parser';

describe('parseGitHubUrl', () => {
  it('correctly parses a standard GitHub URL', () => {
    const result = parseGitHubUrl('https://github.com/facebook/react');
    expect(result).toEqual({ owner: 'facebook', repo: 'react' });
  });
});