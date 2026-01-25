import { useEffect } from 'react'
import type { JSX } from 'react'
import { Card } from './ui/Card'
import { Button } from './ui/Button'
import { X } from 'lucide-react'

interface AboutProps {
  isOpen: boolean
  onClose: () => void
}

export const About = ({ isOpen, onClose }: AboutProps): JSX.Element | null => {
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose()
    }

    if (isOpen) {
      document.addEventListener('keydown', handleKeyDown)
      document.body.style.overflow = 'hidden' // Prevent background scrolling
    }

    return () => {
      document.removeEventListener('keydown', handleKeyDown)
      document.body.style.overflow = 'unset'
    }
  }, [isOpen, onClose])

  if (!isOpen) return null

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm"
      onClick={onClose}
      role="dialog"
      aria-modal="true"
      aria-labelledby="about-title"
    >
      <div onClick={(e) => e.stopPropagation()} className="w-full max-w-2xl">
        <Card className="max-h-[90vh] overflow-y-auto relative animate-in fade-in zoom-in duration-200">
          <button
            onClick={onClose}
            className="absolute top-4 right-4 p-1 hover:bg-bg border-2 border-transparent hover:border-black transition-colors"
            aria-label="Close about modal"
          >
            <X size={24} />
          </button>

          <h2
            id="about-title"
            className="text-2xl md:text-3xl font-black uppercase underline decoration-main decoration-4 mb-6"
          >
            About RepoFlow
          </h2>

          <div className="space-y-6 font-base text-lg">
            <p>
              Traditional metrics like <strong>stars</strong> and <strong>forks</strong> measure
              popularity, but they don't capture the actual <em>velocity</em> of a project.{' '}
              <strong>RepoFlow</strong> fills this gap by measuring the flow of Pull Requests.
            </p>

            <p>A healthy project is defined by a balance between two forces:</p>

            <ul className="list-disc pl-6 space-y-2">
              <li>
                <strong>Contributor Appetite:</strong> A high volume of <em>Opened PRs</em> signals
                that the world is actively using the tool and interested in expanding its utility.
              </li>
              <li>
                <strong>Maintainer Activity:</strong> A high volume of <em>Merged PRs</em> signals
                that maintainers are friendly, responsive, and appropriately managing the interest
                in their project.
              </li>
            </ul>

            <div className="grid gap-4 md:grid-cols-2">
              <div className="border-2 border-black p-4 bg-bg">
                <h3 className="font-heading text-xl mb-2">The Spread</h3>
                <p className="text-sm">
                  The difference between Opened and Merged PRs. A wide spread isn't necessarily bad,
                  but a <em>widening</em> spread is a sign that maintenance is becoming difficult or
                  a backlog is building up.
                </p>
              </div>

              <div className="border-2 border-black p-4 bg-bg">
                <h3 className="font-heading text-xl mb-2">Merge Rate</h3>
                <p className="text-sm">
                  The percentage of opened PRs that were merged. This measures how effectively the
                  project incorporates community contributions.
                </p>
              </div>
            </div>
          </div>

          <div className="mt-8 flex justify-end">
            <Button onClick={onClose}>Got it</Button>
          </div>
        </Card>
      </div>
    </div>
  )
}
