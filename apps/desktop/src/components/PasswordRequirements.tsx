import { Check, Circle } from "lucide-react";
import { getPasswordRequirements } from "@/lib/password";
import { cn } from "@/lib/utils";

export function PasswordRequirements({
  password,
  className,
}: {
  password: string;
  className?: string;
}) {
  const requirements = getPasswordRequirements(password);

  return (
    <ul className={cn("space-y-1.5 text-xs", className)} aria-live="polite">
      {requirements.map((req) => (
        <li
          key={req.id}
          className={cn(
            "flex items-center gap-2 transition-colors",
            req.met ? "text-emerald-600 dark:text-emerald-400" : "text-muted-foreground",
          )}
        >
          {req.met ? (
            <Check className="size-3.5 shrink-0" aria-hidden />
          ) : (
            <Circle className="size-3.5 shrink-0 opacity-50" aria-hidden />
          )}
          <span>{req.label}</span>
        </li>
      ))}
    </ul>
  );
}
