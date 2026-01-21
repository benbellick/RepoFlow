import type { ChangeEvent, JSX } from 'react'
import { cn } from '../../utils/utils'

/**
 * Props for the Input component.
 */
interface InputProps {
  /** The current value of the input. */
  value: string
  /** Handler for change events. */
  onChange: (e: ChangeEvent<HTMLInputElement>) => void
  /** Optional placeholder text. */
  placeholder?: string
  /** Optional additional CSS classes. */
  className?: string
}

/**
 * A styled text input component.
 * Features a thick border and distinct focus state.
 */
export const Input = ({ value, onChange, placeholder, className }: InputProps): JSX.Element => (
  <input
    type="text"
    value={value}
    onChange={onChange}
    placeholder={placeholder}
    className={cn(
      'border-4 border-black bg-white px-4 py-3 font-base text-lg outline-none focus:ring-2 focus:ring-mainAccent transition-all',
      className,
    )}
  />
)
