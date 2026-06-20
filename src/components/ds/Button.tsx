import type { ButtonHTMLAttributes, ReactNode } from "react";
import { Loader2 } from "lucide-react";
import { cn } from "../../lib/cn";

type Variant = "primary" | "secondary" | "ghost" | "danger";
type Size = "sm" | "md";

interface Props extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant;
  size?: Size;
  icon?: ReactNode;
  loading?: boolean;
}

const variants: Record<Variant, string> = {
  primary:
    "border border-[var(--color-accent-hover)] bg-[var(--color-accent)] text-[#17130b] shadow-[var(--shadow-subtle)] hover:bg-[var(--color-accent-hover)] hover:border-[var(--color-accent-hover)]",
  secondary:
    "border border-[var(--color-border-strong)] bg-[var(--color-surface-raised)] text-[var(--color-text-secondary)] shadow-[var(--shadow-subtle)] hover:border-[var(--color-accent)] hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)]",
  ghost:
    "border border-transparent text-[var(--color-text-muted)] hover:border-[var(--color-border)] hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)]",
  danger:
    "border border-[var(--color-danger)]/45 bg-[var(--color-danger-soft)] text-[var(--color-danger)] hover:border-[var(--color-danger)]",
};

const sizes: Record<Size, string> = {
  sm: "h-8 px-2.5 text-xs",
  md: "h-9 px-3 text-sm",
};

export function Button({
  variant = "secondary",
  size = "md",
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
        "inline-flex shrink-0 items-center justify-center gap-2 rounded-[var(--radius-control)] font-semibold transition active:scale-[0.99] disabled:cursor-not-allowed disabled:opacity-45",
        variants[variant],
        sizes[size],
        className,
      )}
      disabled={disabled || loading}
      {...props}
    >
      {loading ? <Loader2 size={15} className="animate-spin" /> : icon}
      {children}
    </button>
  );
}
