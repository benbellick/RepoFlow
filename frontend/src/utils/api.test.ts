import { describe, it, expect, vi, afterEach } from 'vitest'
import { fetchRepoMetrics } from './api'

describe('fetchRepoMetrics', () => {
  const fetchSpy = vi.fn()
  vi.stubGlobal('fetch', fetchSpy)

  afterEach(() => {
    fetchSpy.mockReset()
  })

  it('calls fetch with correct URL', async () => {
    fetchSpy.mockResolvedValue({
      ok: true,
      json: async () => ({}),
    })

    await fetchRepoMetrics('owner', 'repo')

    expect(fetchSpy).toHaveBeenCalledWith('/api/repos/owner/repo/metrics', { signal: undefined })
  })

  it('passes abort signal to fetch', async () => {
    fetchSpy.mockResolvedValue({
      ok: true,
      json: async () => ({}),
    })

    const controller = new AbortController()
    await fetchRepoMetrics('owner', 'repo', controller.signal)

    expect(fetchSpy).toHaveBeenCalledWith(
      expect.stringContaining('/api/repos/owner/repo/metrics'),
      expect.objectContaining({
        signal: controller.signal,
      }),
    )
  })
})
