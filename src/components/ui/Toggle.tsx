import { cn } from "../../lib/cn";

interface Props {
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
  className?: string;
}

/** Переключатель Выкл / Вкл */
export function Toggle({ checked, onChange, disabled, className }: Props) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      disabled={disabled}
      onClick={(e) => {
        e.stopPropagation();
        if (!disabled) onChange(!checked);
      }}
      className={cn(
        "relative inline-flex h-8 w-[4.5rem] shrink-0 items-center rounded-full border transition focus:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-accent)]/50",
        checked
          ? "border-[var(--color-accent)] bg-[var(--color-accent)]"
          : "border-[var(--color-border)] bg-[var(--color-bg-hover)]",
        disabled && "cursor-not-allowed opacity-50",
        className,
      )}
    >
      <span
        className={cn(
          "pointer-events-none absolute top-1 h-6 w-6 rounded-full bg-white shadow transition-transform",
          checked ? "translate-x-[2.35rem]" : "translate-x-1",
        )}
      />
      <span
        className={cn(
          "pointer-events-none w-full text-center text-[11px] font-semibold tracking-wide",
          checked ? "pr-6 text-white" : "pl-6 text-muted",
        )}
      >
        {checked ? "Вкл" : "Выкл"}
      </span>
    </button>
  );
}
