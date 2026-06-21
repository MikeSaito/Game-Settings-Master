import type { ReactNode } from "react";
import { cn } from "../../lib/cn";

export interface SegmentOption<T extends string> {
  value: T;
  label: ReactNode;
  disabled?: boolean;
}

interface Props<T extends string> {
  value: T;
  options: SegmentOption<T>[];
  onChange: (value: T) => void;
  ariaLabel?: string;
  className?: string;
  segmentClassName?: string;
}

export function SegmentControl<T extends string>({
  value,
  options,
  onChange,
  ariaLabel,
  className,
  segmentClassName,
}: Props<T>) {
  return (
    <div
      className={cn(
        "inline-flex max-w-full rounded-[var(--radius-control)] border border-[var(--color-border-strong)] bg-[var(--color-bg-soft)] p-0.5",
        className,
      )}
      role="tablist"
      aria-label={ariaLabel}
    >
      {options.map((option) => {
        const active = option.value === value;
        return (
          <button
            key={option.value}
            type="button"
            role="tab"
            aria-selected={active}
            disabled={option.disabled}
            onClick={() => onChange(option.value)}
            className={cn(
              "min-h-8 min-w-0 flex-1 whitespace-nowrap rounded-[7px] px-2 py-1 text-center text-xs font-semibold leading-tight transition disabled:cursor-not-allowed disabled:opacity-45",
              segmentClassName,
              active
                ? "bg-[var(--color-accent-soft)] text-[var(--color-text)] ring-1 ring-[var(--color-accent)]/55 shadow-[var(--shadow-subtle)]"
                : "text-[var(--color-text-muted)] hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text-secondary)]",
            )}
          >
            {option.label}
          </button>
        );
      })}
    </div>
  );
}
