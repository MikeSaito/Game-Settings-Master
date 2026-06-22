import type {
  InputHTMLAttributes,
  ReactNode,
  SelectHTMLAttributes,
  TextareaHTMLAttributes,
} from "react";
import { cn } from "@/lib/core";

const fieldBase =
  "w-full rounded-[var(--radius-control)] border border-[var(--color-border-strong)] bg-[var(--color-bg-soft)] text-[var(--color-text)] shadow-[var(--shadow-subtle)] placeholder:text-[var(--color-text-faint)] focus:border-[var(--color-accent)]";

interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  icon?: ReactNode;
}

export function Input({ label, icon, className, ...props }: InputProps) {
  return (
    <label className="block">
      {label && (
        <span className="mb-1.5 block text-xs font-medium text-[var(--color-text-muted)]">
          {label}
        </span>
      )}
      <span className="relative block">
        {icon && (
          <span className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-[var(--color-text-faint)]">
            {icon}
          </span>
        )}
        <input
          className={cn(fieldBase, "h-9 px-3 text-sm", icon ? "pl-9" : undefined, className)}
          {...props}
        />
      </span>
    </label>
  );
}

interface SelectProps extends SelectHTMLAttributes<HTMLSelectElement> {
  label?: string;
}

export function Select({ label, className, children, ...props }: SelectProps) {
  return (
    <label className="block">
      {label && (
        <span className="mb-1.5 block text-xs font-medium text-[var(--color-text-muted)]">
          {label}
        </span>
      )}
      <select className={cn(fieldBase, "h-9 px-3 text-sm", className)} {...props}>
        {children}
      </select>
    </label>
  );
}

interface TextareaProps extends TextareaHTMLAttributes<HTMLTextAreaElement> {
  label?: string;
}

export function Textarea({ label, className, ...props }: TextareaProps) {
  return (
    <label className="block">
      {label && (
        <span className="mb-1.5 block text-xs font-medium text-[var(--color-text-muted)]">
          {label}
        </span>
      )}
      <textarea className={cn(fieldBase, "min-h-24 px-3 py-2 text-sm", className)} {...props} />
    </label>
  );
}
