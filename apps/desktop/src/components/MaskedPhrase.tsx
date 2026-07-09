import { useState } from "react";
import { cn } from "@/lib/utils";

export function MaskedPhrase({
  words,
  className,
}: {
  words: string[];
  className?: string;
}) {
  const [revealed, setRevealed] = useState(false);

  return (
    <div
      tabIndex={0}
      role="group"
      aria-label={
        revealed
          ? "Recovery phrase visible"
          : "Recovery phrase hidden. Hover or focus to reveal."
      }
      className={cn(
        "relative rounded-xl border border-border bg-background/60 p-4 outline-none",
        className,
      )}
      onMouseEnter={() => setRevealed(true)}
      onMouseLeave={() => setRevealed(false)}
      onFocus={() => setRevealed(true)}
      onBlur={(e) => {
        if (!e.currentTarget.contains(e.relatedTarget as Node | null)) {
          setRevealed(false);
        }
      }}
    >
      <div
        className="grid grid-cols-3 gap-3 select-none"
        style={{
          filter: revealed ? "none" : "blur(8px)",
          transition: "none",
        }}
        aria-hidden={!revealed}
      >
        {words.map((word, index) => (
          <div key={`${index}-${word}`} className="rounded-md bg-card px-3 py-2 text-sm">
            <span className="mr-2 text-muted-foreground">{index + 1}.</span>
            {word}
          </div>
        ))}
      </div>
      {!revealed && (
        <p className="pointer-events-none absolute inset-0 flex items-center justify-center text-xs text-muted-foreground">
          Hover or focus to reveal
        </p>
      )}
    </div>
  );
}
