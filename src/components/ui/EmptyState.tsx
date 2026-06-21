import type { LucideIcon } from "lucide-react";
import type { ReactNode } from "react";
import { cn } from "../../lib/cn";

interface Props {
  icon: LucideIcon;
  title: string;
  description?: ReactNode;
  action?: ReactNode;
  className?: string;
}

export function EmptyState({ icon: Icon, title, description, action, className }: Props) {
  return (
    <div
      className={cn(
        "flex flex-col items-center justify-center rounded-xl border border-dashed border-[var(--color-border)] bg-[var(--color-bg-card)] px-8 py-16 text-center",
        className,
      )}
    >
      <div className="mb-4 flex h-14 w-14 items-center justify-center rounded-xl bg-[var(--color-bg-hover)]">
        <Icon className="text-muted" size={28} strokeWidth={1.5} />
      </div>
      <h3 className="text-lg font-semibold text-[var(--color-text)]">{title}</h3>
      {description && (
        <p className="mt-2 max-w-md text-sm leading-relaxed text-body">{description}</p>
      )}
      {action && <div className="mt-6">{action}</div>}
    </div>
  );
}
