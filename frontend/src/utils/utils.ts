import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export function isSameRepo(
  repo1: { owner: string; repo: string } | null,
  repo2: { owner: string; repo: string } | null,
) {
  if (!repo1 || !repo2) return false
  return (
    repo1.owner.toLowerCase() === repo2.owner.toLowerCase() &&
    repo1.repo.toLowerCase() === repo2.repo.toLowerCase()
  )
}

