import type { ReactNode } from "react";
import { cn } from "@/lib/core";

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
  /** When true, each segment sizes to its label instead of sharing width equally. */
  sizeToContent?: boolean;
}

export function SegmentControl<T extends string>({
  value,
  options,
  onChange,
  ariaLabel,
  className,
  segmentClassName,
  sizeToContent = false,
}: Props<T>) {
  return (
    <div
      className={cn(
        "inline-flex max-w-full rounded-full border border-[var(--color-border)] bg-[var(--color-bg-soft)] p-0.5",
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
            tabIndex={active ? 0 : -1}
            onKeyDown={(event) => {
              const available = options.filter((item) => !item.disabled);
              const index = available.findIndex(
                (item) => item.value === option.value,
              );
              const next =
                event.key === "ArrowRight" || event.key === "ArrowDown"
                  ? (index + 1) % available.length
                  : event.key === "ArrowLeft" || event.key === "ArrowUp"
                    ? (index + available.length - 1) % available.length
                    : event.key === "Home"
                      ? 0
                      : event.key === "End"
                        ? available.length - 1
                        : -1;
              if (next < 0) return;
              event.preventDefault();
              onChange(available[next].value);
              const tabs =
                event.currentTarget.parentElement?.querySelectorAll<HTMLButtonElement>(
                  '[role="tab"]:not(:disabled)',
                );
              tabs?.[next]?.focus();
            }}
            onClick={() => onChange(option.value)}
            className={cn(
              "min-h-9 whitespace-nowrap rounded-full py-1.5 text-center text-sm font-semibold leading-tight transition disabled:cursor-not-allowed disabled:opacity-45",
              sizeToContent
                ? "flex-none shrink-0 px-3 sm:px-4"
                : "min-w-0 flex-1 px-2",
              segmentClassName,
              active
                ? "bg-[var(--color-secondary-container)] text-[var(--color-on-secondary-container)]"
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
