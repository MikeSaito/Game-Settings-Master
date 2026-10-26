import type { ButtonHTMLAttributes } from "react";
import { cn } from "@/lib/core";

interface Props
  extends Omit<
    ButtonHTMLAttributes<HTMLButtonElement>,
    "onChange" | "onClick"
  > {
  checked: boolean;
  onChange?: (checked: boolean) => void;
  onClick?: ButtonHTMLAttributes<HTMLButtonElement>["onClick"];
}

export function Switch({
  checked,
  onChange,
  onClick,
  className,
  ...props
}: Props) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      onClick={(event) => {
        onClick?.(event);
        if (!event.defaultPrevented) onChange?.(!checked);
      }}
      className={cn(
        "relative h-7 w-12 shrink-0 rounded-full border transition",
        checked
          ? "border-[var(--color-accent)] bg-[var(--color-accent)]"
          : "border-[var(--color-border-strong)] bg-[var(--color-bg-soft)]",
        className,
      )}
      {...props}
    >
      <span
        className={cn(
          "absolute top-1/2 h-5 w-5 -translate-y-1/2 rounded-full transition",
          checked
            ? "left-[23px] bg-[var(--color-on-accent)]"
            : "left-[3px] bg-[var(--color-text)]",
        )}
      />
    </button>
  );
}
