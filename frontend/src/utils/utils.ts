import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'
import type { PopularRepo } from '../types'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

/**
 * Checks if two repository objects refer to the same repo, ignoring case.
 * Handles null/undefined inputs safely.
 */
export function isSameRepo(repo1: PopularRepo | null, repo2: PopularRepo | null): boolean {
  if (repo1 === repo2) return true
  if (!repo1 || !repo2) return false

  return (
    repo1.owner.toLowerCase() === repo2.owner.toLowerCase() &&
    repo1.repo.toLowerCase() === repo2.repo.toLowerCase()
  )
}
