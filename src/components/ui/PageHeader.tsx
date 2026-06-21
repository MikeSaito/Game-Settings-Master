import type { ReactNode } from "react";
import { cn } from "../../lib/cn";

interface Props {
  title: string;
  subtitle?: string;
  meta?: ReactNode;
  actions?: ReactNode;
  className?: string;
}

export function PageHeader({ title, subtitle, meta, actions, className }: Props) {
  return (
    <div className={cn("flex flex-wrap items-start justify-between gap-4", className)}>
      <div className="min-w-0 flex-1">
        <h2 className="text-2xl font-bold tracking-tight text-[var(--color-text)]">{title}</h2>
        {subtitle && (
          <p className="mt-1.5 truncate font-mono text-xs text-muted">{subtitle}</p>
        )}
        {meta && <div className="mt-3 flex flex-wrap gap-2">{meta}</div>}
      </div>
      {actions && <div className="flex shrink-0 flex-wrap gap-2">{actions}</div>}
    </div>
  );
}
