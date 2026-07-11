import { cn } from "@/lib/utils";
import iconUrl from "@/assets/icon.png";

type BrandMarkProps = {
  className?: string;
  alt?: string;
};

/** App icon used as the in-UI brand mark. */
export function BrandMark({ className, alt = "Taurvia" }: BrandMarkProps) {
  return (
    <img
      src={iconUrl}
      alt={alt}
      draggable={false}
      className={cn(
        "shrink-0 rounded-md bg-[#0b0f14] object-contain p-0.5",
        className,
      )}
    />
  );
}
