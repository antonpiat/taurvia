import { useEffect, useRef } from "react";
import { ChevronDown } from "lucide-react";
import { Label } from "@/components/ui/label";
import { cn } from "@/lib/utils";

export type SelectOption<T extends string = string> = {
  value: T;
  label: string;
  description?: string;
};

export function SelectDropdown<T extends string = string>({
  label,
  value,
  options,
  placeholder = "Select…",
  open,
  disabled,
  onOpenChange,
  onChange,
}: {
  label?: string;
  value: T;
  options: Array<SelectOption<T>>;
  placeholder?: string;
  open: boolean;
  disabled?: boolean;
  onOpenChange: (open: boolean) => void;
  onChange: (value: T) => void;
}) {
  const rootRef = useRef<HTMLDivElement>(null);
  const selected = options.find((option) => option.value === value);

  useEffect(() => {
    if (!open) return;
    const onPointerDown = (event: MouseEvent) => {
      if (!rootRef.current?.contains(event.target as Node)) {
        onOpenChange(false);
      }
    };
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") onOpenChange(false);
    };
    document.addEventListener("mousedown", onPointerDown);
    document.addEventListener("keydown", onKeyDown);
    return () => {
      document.removeEventListener("mousedown", onPointerDown);
      document.removeEventListener("keydown", onKeyDown);
    };
  }, [open, onOpenChange]);

  return (
    <div className="space-y-2" ref={rootRef}>
      {label ? <Label>{label}</Label> : null}
      <div className="relative">
        <button
          type="button"
          aria-haspopup="listbox"
          aria-expanded={open}
          disabled={disabled}
          onClick={() => onOpenChange(!open)}
          className={cn(
            "flex h-14 w-full items-center justify-between rounded-md border border-input bg-background px-3 text-left transition-colors",
            "hover:bg-accent/40 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
            "disabled:pointer-events-none disabled:opacity-50",
            open && "ring-2 ring-ring",
          )}
        >
          {selected ? (
            <span className="min-w-0">
              <span className="block text-base font-semibold leading-tight">{selected.label}</span>
              {selected.description ? (
                <span className="block truncate text-xs text-muted-foreground">
                  {selected.description}
                </span>
              ) : null}
            </span>
          ) : (
            <span className="text-muted-foreground">{placeholder}</span>
          )}
          <ChevronDown
            className={cn(
              "h-4 w-4 shrink-0 text-muted-foreground transition-transform",
              open && "rotate-180",
            )}
          />
        </button>

        {open && (
          <div
            role="listbox"
            className="absolute left-0 right-0 top-[calc(100%+0.35rem)] z-30 max-h-64 overflow-y-auto rounded-md border border-border bg-card p-1 shadow-lg"
          >
            {options.map((option) => {
              const isSelected = option.value === value;
              return (
                <button
                  key={option.value}
                  type="button"
                  role="option"
                  aria-selected={isSelected}
                  onClick={() => {
                    onChange(option.value);
                    onOpenChange(false);
                  }}
                  className={cn(
                    "flex w-full flex-col rounded-md px-2.5 py-2 text-left transition-colors",
                    isSelected ? "bg-primary/15" : "hover:bg-accent/50",
                  )}
                >
                  <span className="font-semibold">{option.label}</span>
                  {option.description ? (
                    <span className="text-xs text-muted-foreground">{option.description}</span>
                  ) : null}
                </button>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}
