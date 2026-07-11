import type { ReactNode } from "react";
import { cn } from "@/lib/utils";

export function PageHeader({
  title,
  description,
  parent,
  actions,
  className,
}: {
  title: string;
  description?: ReactNode;
  /** When set, renders `Parent > Title` (e.g. Settings > View). */
  parent?: string;
  actions?: ReactNode;
  className?: string;
}) {
  return (
    <div
      className={cn(
        "flex flex-col gap-3 border-b border-border pb-4 sm:flex-row sm:items-end sm:justify-between sm:pb-5",
        className,
      )}
    >
      <div className="min-w-0 space-y-1.5">
        <h1 className="text-2xl font-semibold tracking-tight text-foreground sm:text-3xl">
          {parent ? (
            <span className="inline-flex flex-wrap items-baseline gap-x-2">
              <span className="text-muted-foreground">{parent}</span>
              <span className="select-none text-border" aria-hidden>
                ›
              </span>
              <span>{title}</span>
            </span>
          ) : (
            title
          )}
        </h1>
        {description ? (
          <p className="max-w-2xl text-sm leading-relaxed text-muted-foreground sm:text-base">
            {description}
          </p>
        ) : null}
      </div>
      {actions ? <div className="flex w-full shrink-0 gap-2 sm:w-auto">{actions}</div> : null}
    </div>
  );
}
