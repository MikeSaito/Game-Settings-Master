import type { InputHTMLAttributes, ReactNode } from "react";
import { cn } from "../../lib/cn";

interface Props extends InputHTMLAttributes<HTMLInputElement> {
  icon?: ReactNode;
  label?: string;
}

export function Input({ icon, label, className, ...props }: Props) {
  return (
    <div className="w-full">
      {label && (
        <label className="mb-1.5 block text-sm font-medium text-[var(--color-text-secondary)]">
          {label}
        </label>
      )}
      <div className="relative">
        {icon && (
          <span className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-[var(--color-text-faint)]">
            {icon}
          </span>
        )}
        <input
          className={cn(
            "input-field w-full rounded-lg py-2.5 text-sm transition",
            icon ? "pl-10 pr-3" : "px-3",
            className,
          )}
          {...props}
        />
      </div>
    </div>
  );
}
