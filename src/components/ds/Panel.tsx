import type { HTMLAttributes, ReactNode } from "react";
import { cn } from "@/lib/core";

interface PanelProps extends HTMLAttributes<HTMLDivElement> {
  children: ReactNode;
  padding?: "none" | "sm" | "md";
}

const padding = {
  none: "",
  sm: "p-3",
  md: "p-4",
};

export function Panel({ children, className, padding: pad = "md", ...props }: PanelProps) {
  return (
    <div
      className={cn(
        "rounded-[var(--radius-panel)] border border-[var(--color-border-strong)] bg-[var(--color-surface)] shadow-[var(--shadow-subtle)]",
        padding[pad],
        className,
      )}
      {...props}
    >
      {children}
    </div>
  );
}

export function Divider({ className }: { className?: string }) {
  return <div className={cn("h-px bg-[var(--color-border-strong)]", className)} />;
}

export function Stack({
  className,
  children,
  gap = "md",
}: {
  className?: string;
  children: ReactNode;
  gap?: "sm" | "md" | "lg";
}) {
  const gaps = { sm: "space-y-2", md: "space-y-3", lg: "space-y-4" };
  return <div className={cn(gaps[gap], className)}>{children}</div>;
}
