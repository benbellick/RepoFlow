import { useState, useEffect, useCallback } from 'react'
import type { JSX, ChangeEvent, FormEvent } from 'react'
import type { FlowMetrics, SummaryMetrics, PopularRepo } from './types'
import { Input } from './components/ui/Input'
import { Button } from './components/ui/Button'
import { FlowChart } from './components/FlowChart'
import { StatCard } from './components/StatsCards'
import { About } from './components/About'
import { TrendDirection } from './types'
import { parseGitHubUrl } from './utils/parser'
import { fetchRepoMetrics, fetchPopularRepos } from './utils/api'
import { Loader2, AlertCircle, Star } from 'lucide-react'

function App(): JSX.Element {
  const [repoUrl, setRepoUrl] = useState<string>('')
  const [data, setData] = useState<FlowMetrics[]>([])
  const [summary, setSummary] = useState<SummaryMetrics | null>(null)
  const [loading, setLoading] = useState<boolean>(true)
  const [error, setError] = useState<string | null>(null)
  const [popularRepos, setPopularRepos] = useState<PopularRepo[]>([])
  const [isAboutOpen, setIsAboutOpen] = useState<boolean>(false)

  const fetchData = useCallback(async (url: string): Promise<void> => {
    const repoDetails = parseGitHubUrl(url)
    if (!repoDetails) {
      setError('Invalid GitHub URL. Please use format: https://github.com/owner/repo')
      return
    }

    setLoading(true)
    setError(null)

    try {
      const response = await fetchRepoMetrics(repoDetails.owner, repoDetails.repo)
      setData(response.time_series)
      setSummary(response.summary)
      setRepoUrl(url)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'An unknown error occurred')
    } finally {
      setLoading(false)
    }
  }, [])

  const handleSearch = (e: FormEvent): void => {
    e.preventDefault()
    fetchData(repoUrl)
  }

  const handlePopularClick = useCallback((owner: string, repo: string): void => {
    const url = `https://github.com/${owner}/${repo}`
    fetchData(url)
  }, [fetchData])

  useEffect(() => {
    const init = async () => {
      try {
        const popular = await fetchPopularRepos()
        setPopularRepos(popular)
        
        if (popular.length > 0) {
          const defaultRepo = popular[0]
          handlePopularClick(defaultRepo.owner, defaultRepo.repo)
        }
      } catch (err) {
        console.error('Failed to load popular repos', err)
        // Fallback to a default if popular fetch fails
        fetchData('https://github.com/facebook/react')
      }
    }
    init()
  }, [fetchData, handlePopularClick])

  const hasData = data.some(day => day.opened > 0 || day.merged > 0)

  return (
    <div className="min-h-screen bg-bg p-8 font-sans selection:bg-main">
      <About isOpen={isAboutOpen} onClose={() => setIsAboutOpen(false)} />
      
      <header className="max-w-7xl mx-auto mb-12 flex flex-col md:flex-row md:items-end justify-between gap-6">
        <div>
          <h1 className="text-6xl mb-2 italic tracking-tighter text-black font-black uppercase underline decoration-main decoration-8">RepoFlow</h1>
          <p className="text-xl font-base">Measure OSS contribution efficiency.</p>
        </div>
        
        <form onSubmit={handleSearch} className="flex gap-4 w-full md:w-auto">
          <Input 
            value={repoUrl} 
            onChange={(e: ChangeEvent<HTMLInputElement>) => setRepoUrl(e.target.value)}
            placeholder="https://github.com/owner/repo"
            className="flex-grow md:w-96"
          />
          <Button type="submit" className="flex items-center gap-2 min-w-[140px] justify-center">
            Analyze
          </Button>
        </form>
      </header>

      <main className="max-w-7xl mx-auto">
        <section className="mb-12">
          <div className="flex items-center gap-2 mb-4">
            <Star className="text-black fill-main" size={24} />
            <h2 className="text-2xl font-black uppercase tracking-tight">Popular Repositories</h2>
          </div>
          <div className="flex flex-wrap gap-3">
            {popularRepos.map((pr) => (
              <button
                key={`${pr.owner}/${pr.repo}`}
                onClick={() => handlePopularClick(pr.owner, pr.repo)}
                className="px-4 py-2 border-2 border-black bg-white font-heading hover:bg-main hover:translate-x-[-2px] hover:translate-y-[-2px] hover:shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] active:translate-x-[0px] active:translate-y-[0px] active:shadow-none transition-all"
              >
                {pr.owner}/{pr.repo}
              </button>
            ))}
            {popularRepos.length === 0 && !loading && (
              <p className="italic text-gray-600">No popular repos loaded.</p>
            )}
          </div>
        </section>

        {error && (
          <div className="mb-8 border-4 border-black bg-red-400 p-4 flex items-center gap-3 font-heading shadow-[8px_8px_0px_0px_rgba(0,0,0,1)]">
            <AlertCircle size={24} />
            {error}
          </div>
        )}

        {summary && (
          <div className={`grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8 transition-opacity duration-300 ${loading ? 'opacity-50' : 'opacity-100'}`}>
            <StatCard
              label="PRs Opened (30d)"
              value={summary.current_opened}
              color="bg-white"
              description="Total pull requests created in the last 30 days."
            />
            <StatCard
              label="PRs Merged (30d)"
              value={summary.current_merged}
              color="bg-white"
              description="Total pull requests merged in the last 30 days."
            />
            <StatCard
              label="The Spread"
              value={summary.current_spread}
              trend={summary.is_widening ? TrendDirection.UP : TrendDirection.DOWN}
              trendLabel={summary.is_widening ? 'Widening' : 'Tightening'}
              color="bg-main"
              description="Difference between opened and merged PRs. A widening spread means the backlog is growing."
            />
            <StatCard
              label="Merge Rate"
              value={`${summary.merge_rate}%`}
              color="bg-white"
              description="Percentage of opened PRs that were merged in the same period."
            />
          </div>
        )}

        <div className={`transition-opacity duration-300 ${loading ? 'opacity-50' : 'opacity-100'}`}>
          {loading && data.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-20 border-4 border-dashed border-black rounded-none bg-white shadow-[8px_8px_0px_0px_rgba(0,0,0,1)]">
               <Loader2 className="animate-spin mb-4" size={48} />
               <p className="font-heading text-xl uppercase tracking-widest">Fetching repo history...</p>
            </div>
          ) : hasData ? (
            <FlowChart data={data} />
          ) : (
            <div className="border-4 border-black bg-white p-12 text-center shadow-base">
              <h2 className="text-3xl font-heading mb-4">No Data Found</h2>
              <p className="text-xl font-base text-gray-600">
                We couldn't find any pull requests for this repository in the analyzed time period.
              </p>
            </div>
          )}
        </div>
      </main>

      <footer className="max-w-7xl mx-auto mt-12 pt-8 border-t-4 border-black font-base flex justify-between items-center">
        <p>RepoFlow - Measuring PR Liquidity</p>
        <div className="flex gap-6">
          <a href="https://github.com/benbellick/RepoFlow" className="hover:underline font-heading">GitHub</a>
          <button onClick={() => setIsAboutOpen(true)} className="hover:underline font-heading hover:bg-main px-2 -mx-2 transition-colors">About</button>
        </div>
      </footer>
    </div>
  )
}

export default App
