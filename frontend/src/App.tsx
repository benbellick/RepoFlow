import { useState, useEffect } from 'react'
import type { JSX, ChangeEvent, FormEvent } from 'react'
import type { FlowMetrics } from './types'
import { Input } from './components/ui/Input'
import { Button } from './components/ui/Button'
import { FlowChart } from './components/FlowChart'
import { StatCard } from './components/StatsCards'
import { TrendDirection } from './types'
import { parseGitHubUrl } from './utils/parser'
import { fetchRepoMetrics } from './utils/api'
import { Loader2, AlertCircle } from 'lucide-react'

function App(): JSX.Element {
  const [repoUrl, setRepoUrl] = useState<string>('https://github.com/facebook/react')
  const [data, setData] = useState<FlowMetrics[]>([])
  const [loading, setLoading] = useState<boolean>(true)
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
      const metrics = await fetchRepoMetrics(repoDetails.owner, repoDetails.repo)
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

  const currentOpened = data.length > 0 ? data[data.length - 1].opened : 0
  const currentMerged = data.length > 0 ? data[data.length - 1].merged : 0
  const currentSpread = data.length > 0 ? data[data.length - 1].spread : 0
  const mergeRate = currentOpened > 0 ? Math.round((currentMerged / currentOpened) * 100) : 0

  // Trend analysis
  const prevSpread = data.length > 1 ? data[data.length - 2].spread : currentSpread
  const isWidening = currentSpread > prevSpread
  
  return (
    <div className="min-h-screen bg-bg p-8 font-sans selection:bg-main">
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
          <Button type="submit" className="flex items-center gap-2">
            {loading ? <Loader2 className="animate-spin" size={20} /> : 'Analyze'}
          </Button>
        </form>
      </header>

      <main className="max-w-7xl mx-auto">
        {error && (
          <div className="mb-8 border-4 border-black bg-red-400 p-4 flex items-center gap-3 font-heading shadow-[8px_8px_0px_0px_rgba(0,0,0,1)]">
            <AlertCircle size={24} />
            {error}
          </div>
        )}

        {data.length > 0 && (
          <>
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
          </>
        )}

        {loading && data.length === 0 && (
          <div className="flex flex-col items-center justify-center py-20 border-4 border-dashed border-black rounded-none bg-white shadow-[8px_8px_0px_0px_rgba(0,0,0,1)]">
            <Loader2 className="animate-spin mb-4" size={48} />
            <p className="font-heading text-xl uppercase tracking-widest">Fetching repo history...</p>
          </div>
        )}
      </main>

      <footer className="max-w-7xl mx-auto mt-12 pt-8 border-t-4 border-black font-base flex justify-between items-center">
        <p>© 2026 RepoFlow - Measuring PR Liquidity</p>
        <div className="flex gap-6">
          <a href="https://github.com/benbellick/RepoFlow" className="hover:underline font-heading">GitHub</a>
          <a href="#" className="hover:underline font-heading">About</a>
        </div>
      </footer>
    </div>
  )
}

export default App