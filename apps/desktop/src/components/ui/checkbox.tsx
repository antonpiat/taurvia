import * as React from "react";
import { cn } from "@/lib/utils";

export interface CheckboxProps
  extends Omit<React.InputHTMLAttributes<HTMLInputElement>, "type" | "onChange"> {
  checked: boolean;
  onCheckedChange: (checked: boolean) => void;
}

/**
 * Custom checkbox — replaces native OS control (thick/off-center check, harsh accent fill).
 * Uses a real input for label association + keyboard; paints a thin centered mark.
 */
export const Checkbox = React.forwardRef<HTMLInputElement, CheckboxProps>(
  ({ checked, onCheckedChange, className, disabled, id, ...props }, ref) => {
    return (
      <span className={cn("relative mt-0.5 inline-flex h-4 w-4 shrink-0", className)}>
        <input
          ref={ref}
          id={id}
          type="checkbox"
          checked={checked}
          disabled={disabled}
          onChange={(e) => onCheckedChange(e.target.checked)}
          className="peer absolute inset-0 z-10 h-full w-full cursor-pointer opacity-0 disabled:cursor-not-allowed"
          {...props}
        />
        <span
          aria-hidden
          className={cn(
            "pointer-events-none flex h-4 w-4 items-center justify-center rounded-[3px] border transition-colors",
            "border-input bg-transparent text-transparent",
            "peer-hover:border-muted-foreground/50",
            "peer-focus-visible:ring-2 peer-focus-visible:ring-ring peer-focus-visible:ring-offset-2 peer-focus-visible:ring-offset-background",
            "peer-checked:border-primary peer-checked:bg-primary peer-checked:text-primary-foreground",
            "peer-disabled:opacity-50",
          )}
        >
          <svg viewBox="0 0 16 16" className="h-2.5 w-2.5" fill="none">
            <path
              d="M3.5 8.25L6.6 11.25L12.5 4.75"
              stroke="currentColor"
              strokeWidth="1.5"
              strokeLinecap="round"
              strokeLinejoin="round"
              vectorEffect="non-scaling-stroke"
            />
          </svg>
        </span>
      </span>
    );
  },
);
Checkbox.displayName = "Checkbox";
