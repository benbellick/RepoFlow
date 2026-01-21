import type { ReactNode, JSX } from "react";
import { cn } from "../../utils/utils";

/**
 * Props for the Card component.
 */
interface CardProps {
  /** The content to be rendered inside the card. */
  children: ReactNode;
  /** Optional additional CSS classes. */
  className?: string;
}

/**
 * A generic container component with neo-brutalism styling.
 * Renders a white box with a thick black border and hard shadow.
 */
export const Card = ({ children, className }: CardProps): JSX.Element => (
  <div
    className={cn("border-4 border-black bg-white p-6 shadow-base", className)}
  >
    {children}
  </div>
);
