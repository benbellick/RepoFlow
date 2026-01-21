import type { ReactNode, JSX } from "react";
import { cn } from "../../utils/utils";

/**
 * Props for the Button component.
 */
interface ButtonProps {
  /** The content to be rendered inside the button. */
  children: ReactNode;
  /** Optional click handler. */
  onClick?: () => void;
  /** Optional additional CSS classes. */
  className?: string;
  /** The type of the button. Defaults to "button". */
  type?: "button" | "submit";
  /** Whether the button is disabled. */
  disabled?: boolean;
}

/**
 * A neo-brutalism styled button component.
 * Features a thick border, hard shadow, and press animation.
 */
export const Button = ({
  children,
  onClick,
  className,
  type = "button",
  disabled,
}: ButtonProps): JSX.Element => (
  <button
    type={type}
    onClick={onClick}
    disabled={disabled}
    className={cn(
      "border-4 border-black bg-main px-6 py-3 font-heading text-xl shadow-base active:translate-x-1 active:translate-y-1 active:shadow-none transition-all disabled:opacity-50 disabled:pointer-events-none",
      className,
    )}
  >
    {children}
  </button>
);
