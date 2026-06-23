import type { ReactNode } from "react";

interface Props {
  title: string;
  description?: ReactNode;
  hint?: ReactNode;
}

export function BackupSectionTitle({ title, description, hint }: Props) {
  return (
    <div className="mb-4 flex flex-wrap items-end justify-between gap-2">
      <div className="min-w-0">
        <h3 className="text-sm font-semibold uppercase tracking-wide text-[var(--color-text-secondary)]">
          {title}
        </h3>
        {description && (
          <p className="mt-1 text-sm text-[var(--color-text-muted)]">{description}</p>
        )}
      </div>
      {hint && <div className="text-sm text-[var(--color-text-muted)]">{hint}</div>}
    </div>
  );
}
