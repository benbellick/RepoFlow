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
            Traditional metrics like <strong>stars</strong> and <strong>forks</strong> measure popularity, but they don't capture the actual <em>velocity</em> of a project. 
            <strong>RepoFlow</strong> fills this gap by measuring the flow of Pull Requests.
          </p>

          <p>
            A healthy project is defined by a balance between two forces:
          </p>

          <ul className="list-disc pl-6 space-y-2">
            <li>
              <strong>Contributor Appetite:</strong> A high volume of <em>Opened PRs</em> signals that the world is hungry to improve the tool.
            </li>
            <li>
              <strong>Maintainer Activity:</strong> A high volume of <em>Merged PRs</em> signals a friendly, responsive maintainer team.
            </li>
          </ul>

          <div className="grid gap-4 md:grid-cols-2">
            <div className="border-2 border-black p-4 bg-bg">
              <h3 className="font-heading text-xl mb-2">The Spread</h3>
              <p className="text-sm">
                The difference between Opened and Merged PRs. 
                Health is about these two matching comparably. A wide spread suggests a project that is "frozen" or underwater, while a tight spread indicates "liquidity."
              </p>
            </div>

            <div className="border-2 border-black p-4 bg-bg">
              <h3 className="font-heading text-xl mb-2">Merge Rate</h3>
              <p className="text-sm">
                The percentage of opened PRs that were merged. 
                This measures the efficiency of the conversion from "stated appetite" to "actual project growth."
              </p>
            </div>
          </div>
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
