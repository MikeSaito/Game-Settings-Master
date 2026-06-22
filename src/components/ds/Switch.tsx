import type { ButtonHTMLAttributes } from "react";
import { cn } from "@/lib/core";

interface Props extends Omit<ButtonHTMLAttributes<HTMLButtonElement>, "onChange" | "onClick"> {
  checked: boolean;
  onChange?: (checked: boolean) => void;
  onClick?: ButtonHTMLAttributes<HTMLButtonElement>["onClick"];
}

export function Switch({ checked, onChange, onClick, className, ...props }: Props) {
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
        "relative h-5 w-9 shrink-0 rounded-full border transition",
        checked
          ? "border-[var(--color-accent)] bg-[var(--color-accent)]"
          : "border-[var(--color-border-strong)] bg-[var(--color-bg-soft)]",
        className,
      )}
      {...props}
    >
      <span
        className={cn(
          "absolute top-1/2 h-3.5 w-3.5 -translate-y-1/2 rounded-full bg-[var(--color-text)] transition",
          checked ? "left-[18px] bg-[#17130b]" : "left-[3px]",
        )}
      />
    </button>
  );
}
