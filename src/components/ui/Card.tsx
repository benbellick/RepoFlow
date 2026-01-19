import type { ReactNode, JSX } from 'react'
import { cn } from '../../utils/utils'

interface CardProps {
  children: ReactNode
  className?: string
}

export const Card = ({ children, className }: CardProps): JSX.Element => (
  <div className={cn("border-4 border-black bg-white p-6 shadow-base", className)}>
    {children}
  </div>
)