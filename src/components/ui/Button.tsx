import type { ReactNode, JSX } from 'react'
import { cn } from '../../utils/utils'

interface ButtonProps {
  children: ReactNode
  onClick?: () => void
  className?: string
  type?: "button" | "submit"
}

export const Button = ({ children, onClick, className, type = "button" }: ButtonProps): JSX.Element => (
  <button 
    type={type}
    onClick={onClick}
    className={cn(
      "border-4 border-black bg-main px-6 py-3 font-heading text-xl shadow-base active:translate-x-1 active:translate-y-1 active:shadow-none transition-all",
      className
    )}
  >
    {children}
  </button>
)