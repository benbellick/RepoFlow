import type { ChangeEvent, JSX } from 'react'
import { cn } from '../../utils/utils'

interface InputProps {
  value: string
  onChange: (e: ChangeEvent<HTMLInputElement>) => void
  placeholder?: string
  className?: string
}

export const Input = ({ value, onChange, placeholder, className }: InputProps): JSX.Element => (
  <input
    type="text"
    value={value}
    onChange={onChange}
    placeholder={placeholder}
    className={cn(
      "border-4 border-black bg-white px-4 py-3 font-base text-lg outline-none focus:ring-2 focus:ring-mainAccent transition-all",
      className
    )}
  />
)