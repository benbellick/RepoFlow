import { describe, it, expect, vi } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/react';
import App from './App';
import * as github from './utils/github';

describe('App', () => {
  it('shows no data message when metrics are empty', async () => {
    const fetchSpy = vi.spyOn(github, 'fetchPullRequests').mockResolvedValue([]);

    render(<App />);

    const input = screen.getByPlaceholderText('https://github.com/owner/repo');
    const button = await screen.findByText('Analyze');

    fireEvent.change(input, { target: { value: 'https://github.com/empty/repo' } });
    fireEvent.click(button);

    await waitFor(() => {
        expect(fetchSpy).toHaveBeenCalledWith('empty', 'repo');
    });

    const noDataMessage = await screen.findByText(/No data found/i);
    expect(noDataMessage).toBeTruthy();

    // StatCards should still be showing
    const statLabel = screen.queryByText(/PRs Opened/i);
    expect(statLabel).toBeTruthy();
  });
});
