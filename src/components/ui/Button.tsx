import type { ButtonHTMLAttributes, ReactNode } from "react";
import { cn } from "@/lib/core";

type Variant = "primary" | "secondary" | "ghost" | "danger";

interface Props extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant;
  icon?: ReactNode;
  loading?: boolean;
}

const variants: Record<Variant, string> = {
  primary:
    "border border-[var(--color-accent-hover)] bg-[var(--color-accent)] text-[#17130b] shadow-[var(--shadow-subtle)] hover:bg-[var(--color-accent-hover)]",
  secondary:
    "border border-[var(--color-border-strong)] bg-[var(--color-surface-raised)] text-[var(--color-text-secondary)] shadow-[var(--shadow-subtle)] hover:border-[var(--color-accent)] hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)]",
  ghost: "text-muted hover:text-[var(--color-text)] hover:bg-[var(--color-bg-hover)]",
  danger:
    "border border-[#5a3030] bg-[#2e1a1a] text-[#f0a0a0] hover:bg-[#3a2020]",
};

export function Button({
  variant = "secondary",
  icon,
  loading,
  className,
  children,
  disabled,
  ...props
}: Props) {
  return (
    <button
      type="button"
      className={cn(
        "inline-flex items-center justify-center gap-2 rounded-lg px-4 py-2.5 text-sm font-medium transition",
        variants[variant],
        className,
      )}
      disabled={disabled || loading}
      {...props}
    >
      {loading ? (
        <span className="h-4 w-4 animate-spin rounded-full border-2 border-white/25 border-t-white" />
      ) : (
        icon
      )}
      {children}
    </button>
  );
}
