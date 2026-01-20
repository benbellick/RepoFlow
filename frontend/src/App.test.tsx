import { describe, it, expect, vi } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/react';
import App from './App';
import * as github from './utils/github';

describe('App', () => {
  it('shows no data message when metrics are empty', async () => {
    // Mock fetchPullRequests to return empty array
    const fetchSpy = vi.spyOn(github, 'fetchPullRequests').mockResolvedValue([]);

    // We need to render the App
    render(<App />);

    // The App calls fetchData on mount for 'facebook/react'.
    // We want to verify what happens when that returns empty.

    // However, App.tsx useEffect calls fetchData with 'facebook/react'.
    // If our mock returns [], calculateMetrics will return 30 days of 0s.

    // Let's trigger a search for a repo that returns empty data
    const input = screen.getByPlaceholderText('https://github.com/owner/repo');
    // Button text changes to "Analyze" when not loading.
    // Wait for the button to have text 'Analyze'
    const button = await screen.findByText('Analyze');

    fireEvent.change(input, { target: { value: 'https://github.com/empty/repo' } });
    fireEvent.click(button);

    await waitFor(() => {
        expect(fetchSpy).toHaveBeenCalledWith('empty', 'repo');
    });

    // Now check if "No data found" is displayed.
    const noDataMessage = await screen.findByText(/No data found/i);
    expect(noDataMessage).toBeTruthy();

    // We can also check that StatCards are NOT showing
    // The stats cards have "PRs Opened (30d)", etc.
    const statLabel = screen.queryByText(/PRs Opened/i);
    expect(statLabel).toBeNull();
  });
});
