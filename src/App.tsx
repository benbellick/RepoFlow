import { useState, useEffect } from 'react'
import type { JSX, ChangeEvent, FormEvent } from 'react'
import { calculateMetrics, generateDummyData } from './utils/metrics'
import type { FlowMetrics } from './utils/metrics'
import { Input } from './components/ui/Input'
import { Button } from './components/ui/Button'
import { FlowChart } from './components/FlowChart'
import { StatCard } from './components/StatsCards'
import { TrendDirection } from './types'
import { parseGitHubUrl } from './utils/parser'
import { fetchPullRequests } from './utils/github'
import { Loader2, AlertCircle } from 'lucide-react'

function App(): JSX.Element {
  const [repoUrl, setRepoUrl] = useState<string>('https://github.com/facebook/react')
  const [data, setData] = useState<FlowMetrics[]>(generateDummyData(30))
  const [loading, setLoading] = useState<boolean>(false)
  const [error, setError] = useState<string | null>(null)

  const fetchData = async (url: string): Promise<void> => {
    const repoDetails = parseGitHubUrl(url)
    if (!repoDetails) {
      setError('Invalid GitHub URL. Please use format: https://github.com/owner/repo')
      return
    }

    setLoading(true)
    setError(null)

    try {
      const prs = await fetchPullRequests(repoDetails.owner, repoDetails.repo)
      const metrics = calculateMetrics(prs)
      setData(metrics)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'An unknown error occurred')
    } finally {
      setLoading(false)
    }
  }

  const handleSearch = (e: FormEvent): void => {
    e.preventDefault()
    fetchData(repoUrl)
  }

  // Load initial data for React
  useEffect(() => {
    fetchData('https://github.com/facebook/react')
  }, [])

  const currentOpened = data[data.length - 1].opened
  const currentMerged = data[data.length - 1].merged
  const currentSpread = data[data.length - 1].spread
  const mergeRate = currentOpened > 0 ? Math.round((currentMerged / currentOpened) * 100) : 0

  // Trend analysis (very simple V1)
  const prevSpread = data.length > 1 ? data[data.length - 2].spread : currentSpread
  const isWidening = currentSpread > prevSpread
  
  return (
    <div className="min-h-screen bg-bg p-8 font-sans selection:bg-main">
      <header className="max-w-7xl mx-auto mb-12 flex flex-col md:flex-row md:items-end justify-between gap-6">
        <div>
          <h1 className="text-6xl mb-2 italic tracking-tighter">RepoFlow</h1>
          <p className="text-xl font-base">Measure OSS contribution efficiency.</p>
        </div>
        
        <form onSubmit={handleSearch} className="flex gap-4 w-full md:w-auto">
          <Input 
            value={repoUrl} 
            onChange={(e: ChangeEvent<HTMLInputElement>) => setRepoUrl(e.target.value)}
            placeholder="https://github.com/owner/repo"
            className="flex-grow md:w-96"
          />
          <Button type="submit" className="flex items-center gap-2">
            {loading ? <Loader2 className="animate-spin" size={20} /> : 'Analyze'}
          </Button>
        </form>
      </header>

      <main className="max-w-7xl mx-auto">
        {error && (
          <div className="mb-8 border-4 border-black bg-red-400 p-4 flex items-center gap-3 font-heading">
            <AlertCircle size={24} />
            {error}
          </div>
        )}

        <div className={`grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8 transition-opacity duration-300 ${loading ? 'opacity-50' : 'opacity-100'}`}>
          <StatCard 
            label="PRs Opened (30d)" 
            value={currentOpened} 
            color="bg-white"
          />
          <StatCard 
            label="PRs Merged (30d)" 
            value={currentMerged} 
            color="bg-white"
          />
          <StatCard 
            label="The Spread" 
            value={currentSpread} 
            trend={isWidening ? TrendDirection.UP : TrendDirection.DOWN} 
            trendLabel={isWidening ? 'Widening' : 'Tightening'}
            color="bg-main"
          />
          <StatCard 
            label="Merge Rate" 
            value={`${mergeRate}%`} 
            color="bg-white"
          />
        </div>

        <div className={`transition-opacity duration-300 ${loading ? 'opacity-50' : 'opacity-100'}`}>
          <FlowChart data={data} />
        </div>
      </main>

      <footer className="max-w-7xl mx-auto mt-12 pt-8 border-t-4 border-black font-base flex justify-between items-center">
        <p>© 2026 RepoFlow - Measuring PR Liquidity</p>
        <div className="flex gap-6">
          <a href="https://github.com" className="hover:underline font-heading">GitHub</a>
          <a href="#" className="hover:underline font-heading">About</a>
        </div>
      </footer>
    </div>
  )
}

export default App
