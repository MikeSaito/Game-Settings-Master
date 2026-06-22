import type { ElementType, ReactNode } from "react";
import { Info, Search } from "lucide-react";
import { cn } from "@/lib/core";
import { Button } from "./Button";

type Tone = "info" | "success" | "warning" | "danger";

const tones: Record<Tone, string> = {
  info: "border-[var(--color-border)] bg-[var(--color-surface)] text-[var(--color-text-secondary)]",
  success: "border-[var(--color-success)]/35 bg-[var(--color-success-soft)] text-[var(--color-success)]",
  warning: "border-[var(--color-warning)]/35 bg-[var(--color-warning-soft)] text-[var(--color-warning)]",
  danger: "border-[var(--color-danger)]/35 bg-[var(--color-danger-soft)] text-[var(--color-danger)]",
};

export function Alert({
  tone = "info",
  icon: Icon = Info,
  title,
  children,
  className,
}: {
  tone?: Tone;
  icon?: ElementType;
  title?: ReactNode;
  children: ReactNode;
  className?: string;
}) {
  return (
    <div className={cn("flex gap-2 rounded-[var(--radius-control)] border px-3 py-2 text-sm", tones[tone], className)}>
      <Icon size={16} className="mt-0.5 shrink-0" />
      <div className="min-w-0">
        {title && <div className="font-semibold text-[var(--color-text)]">{title}</div>}
        <div className="text-[var(--color-text-secondary)]">{children}</div>
      </div>
    </div>
  );
}

export function EmptyState({
  icon: Icon = Search,
  title,
  description,
  primaryAction,
  secondaryAction,
  className,
}: {
  icon?: ElementType;
  title: ReactNode;
  description?: ReactNode;
  primaryAction?: ReactNode;
  secondaryAction?: ReactNode;
  className?: string;
}) {
  return (
    <div className={cn("flex flex-col items-center justify-center rounded-[var(--radius-panel)] border border-dashed border-[var(--color-border)] bg-[var(--color-bg-soft)] px-6 py-12 text-center", className)}>
      <div className="mb-4 grid h-14 w-14 place-items-center rounded-[var(--radius-panel)] border border-[var(--color-border)] bg-[var(--color-surface)] text-[var(--color-accent)]">
        <Icon size={24} />
      </div>
      <h2 className="text-base font-semibold text-[var(--color-text)]">{title}</h2>
      {description && (
        <p className="mt-2 max-w-md text-sm text-[var(--color-text-muted)]">{description}</p>
      )}
      {(primaryAction || secondaryAction) && (
        <div className="mt-5 flex flex-wrap justify-center gap-2">
          {primaryAction}
          {secondaryAction}
        </div>
      )}
    </div>
  );
}

export function Skeleton({ className }: { className?: string }) {
  return <div className={cn("animate-pulse rounded-[var(--radius-control)] bg-[var(--color-surface-raised)]", className)} />;
}

export function EmptyAction({ children, ...props }: Parameters<typeof Button>[0]) {
  return <Button {...props}>{children}</Button>;
}
