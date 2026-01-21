import type { JSX } from 'react'
import { Card } from './ui/Card'
import { Button } from './ui/Button'
import { X } from 'lucide-react'

interface AboutProps {
  isOpen: boolean
  onClose: () => void
}

export const About = ({ isOpen, onClose }: AboutProps): JSX.Element | null => {
  if (!isOpen) return null

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm">
      <Card className="w-full max-w-2xl max-h-[90vh] overflow-y-auto relative animate-in fade-in zoom-in duration-200">
        <button 
          onClick={onClose}
          className="absolute top-4 right-4 p-1 hover:bg-bg border-2 border-transparent hover:border-black transition-colors"
          aria-label="Close about modal"
        >
          <X size={24} />
        </button>

        <h2 className="text-3xl font-black uppercase underline decoration-main decoration-4 mb-6">
          About RepoFlow
        </h2>

        <div className="space-y-6 font-base text-lg">
          <p>
            <strong>RepoFlow</strong> is a tool designed to visualize the "liquidity" of open source contributions. 
            It helps maintainers and contributors understand how efficiently a repository processes Pull Requests.
          </p>

          <div className="grid gap-4 md:grid-cols-2">
            <div className="border-2 border-black p-4 bg-bg">
              <h3 className="font-heading text-xl mb-2">The Spread</h3>
              <p className="text-sm">
                The difference between <strong>Opened</strong> and <strong>Merged</strong> PRs. 
                A widening positive spread indicates a growing backlog (technical debt), while a tightening or negative spread means the maintainers are catching up.
              </p>
            </div>

            <div className="border-2 border-black p-4 bg-bg">
              <h3 className="font-heading text-xl mb-2">Merge Rate</h3>
              <p className="text-sm">
                The percentage of opened PRs that were eventually merged in the given window. 
                High merge rates suggest a healthy, active project where contributions are welcomed and processed quickly.
              </p>
            </div>
          </div>

          <p>
            This project uses the GitHub API to fetch real-time data. 
            Popular repositories are preloaded for instant access.
          </p>
        </div>

        <div className="mt-8 flex justify-end">
          <Button onClick={onClose}>
            Got it
          </Button>
        </div>
      </Card>
    </div>
  )
}
