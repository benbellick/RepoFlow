import type { JSX } from 'react'
import type { PopularRepo } from '../types'

interface PopularRepoChipProps {
  repo: PopularRepo
  isActive: boolean
  onClick: (repo: PopularRepo) => void
}

export const PopularRepoChip = ({
  repo,
  isActive,
  onClick,
}: PopularRepoChipProps): JSX.Element => {
  return (
    <button
      onClick={() => onClick(repo)}
      className={`px-4 py-2 border-2 border-black font-heading transition-all active:translate-x-0 active:translate-y-0 active:shadow-none ${
        isActive
          ? 'bg-main translate-x-[-2px] translate-y-[-2px] shadow-base'
          : 'bg-white hover:bg-main hover:translate-x-[-2px] hover:translate-y-[-2px] hover:shadow-base'
      }`}
    >
      {repo.owner}/{repo.repo}
    </button>
  )
}

