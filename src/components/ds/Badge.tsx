import type { HTMLAttributes, ReactNode } from "react";
import { cn } from "../../lib/cn";

type Tone = "neutral" | "accent" | "success" | "warning" | "danger" | "info";

interface Props extends HTMLAttributes<HTMLSpanElement> {
  tone?: Tone;
  children: ReactNode;
}

const tones: Record<Tone, string> = {
  neutral: "border-[var(--color-border-strong)] bg-[var(--color-surface)] text-[var(--color-text-secondary)]",
  accent: "border-[var(--color-accent)]/65 bg-[var(--color-accent-soft)] text-[var(--color-accent-hover)]",
  success: "border-[var(--color-success)]/60 bg-[var(--color-success-soft)] text-[var(--color-success)]",
  warning: "border-[var(--color-warning)]/65 bg-[var(--color-warning-soft)] text-[var(--color-warning)]",
  danger: "border-[var(--color-danger)]/60 bg-[var(--color-danger-soft)] text-[var(--color-danger)]",
  info: "border-[var(--color-border-strong)] bg-[var(--color-surface-raised)] text-[var(--color-text-secondary)]",
};

export function Badge({ tone = "neutral", className, children, ...props }: Props) {
  return (
    <span
      className={cn(
        "inline-flex items-center rounded-[6px] border px-1.5 py-0.5 text-[11px] font-medium leading-4",
        tones[tone],
        className,
      )}
      {...props}
    >
      {children}
    </span>
  );
}

export const Chip = Badge;
