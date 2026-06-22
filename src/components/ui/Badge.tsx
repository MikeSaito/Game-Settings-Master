import type { ReactNode } from "react";
import { cn } from "@/lib/core";

type Tone = "default" | "success" | "warning" | "danger" | "info" | "accent";

interface Props {
  children: ReactNode;
  tone?: Tone;
  className?: string;
}

const tones: Record<Tone, string> = {
  default: "bg-[#212733] text-[var(--color-text-secondary)] border-[var(--color-border)]",
  success: "bg-[#1a2e24] text-[#8fd9a8] border-[#2d5a40]",
  warning: "bg-[#2e2618] text-[#e8c468] border-[#5a4a28]",
  danger: "bg-[#2e1a1a] text-[#f0a0a0] border-[#5a3030]",
  info: "bg-[#1a2438] text-[#a8c4ff] border-[#304870]",
  accent: "bg-[var(--color-accent-soft)] text-[#b8d0ff] border-[#3d5a8a]",
};

export function Badge({ children, tone = "default", className }: Props) {
  return (
    <span
      className={cn(
        "inline-flex items-center rounded-md border px-2 py-0.5 text-xs font-medium",
        tones[tone],
        className,
      )}
    >
      {children}
    </span>
  );
}
