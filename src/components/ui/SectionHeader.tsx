import type { ReactNode } from "react";
import { cn } from "../../lib/cn";

interface Props {
  step?: number;
  title: string;
  description?: ReactNode;
  hint?: ReactNode;
  className?: string;
}

export function SectionHeader({ step, title, description, hint, className }: Props) {
  return (
    <div className={cn("mb-4 flex flex-wrap items-end justify-between gap-2", className)}>
      <div className="min-w-0">
        <div className="flex items-center gap-3">
          {step != null && (
            <span className="flex h-7 w-7 items-center justify-center rounded-md bg-[var(--color-accent-soft)] text-xs font-bold text-accent">
              {step}
            </span>
          )}
          <h3 className="text-sm font-semibold uppercase tracking-wide text-[var(--color-text-secondary)]">
            {title}
          </h3>
        </div>
        {description && <p className="mt-1 text-sm text-muted">{description}</p>}
      </div>
      {hint && <div className="text-sm text-muted">{hint}</div>}
    </div>
  );
}
