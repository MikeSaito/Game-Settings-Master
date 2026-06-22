import type { LucideIcon } from "lucide-react";
import type { ReactNode } from "react";
import { cn } from "@/lib/core";

type Tone = "info" | "success" | "warning" | "error";

interface Props {
  tone?: Tone;
  icon?: LucideIcon;
  title?: string;
  children: ReactNode;
  className?: string;
}

const styles: Record<Tone, string> = {
  info: "border-[#304870] bg-[#1a2438] text-[#c8daff]",
  success: "border-[#2d5a40] bg-[#1a2e24] text-[#c8efd4]",
  warning: "border-[#5a4a28] bg-[#2e2618] text-[#f5e6b8]",
  error: "border-[#5a3030] bg-[#2e1a1a] text-[#f5c8c8]",
};

const iconColors: Record<Tone, string> = {
  info: "text-[#8ab4ff]",
  success: "text-[#6fcf97]",
  warning: "text-[#e8c468]",
  error: "text-[#f08080]",
};

export function Alert({ tone = "info", icon: Icon, title, children, className }: Props) {
  return (
    <div
      className={cn(
        "flex items-start gap-3 rounded-xl border p-4",
        styles[tone],
        className,
      )}
    >
      {Icon && <Icon className={cn("mt-0.5 shrink-0", iconColors[tone])} size={20} />}
      <div className="min-w-0 flex-1 text-sm leading-relaxed">
        {title && <div className="mb-1 font-semibold text-[var(--color-text)]">{title}</div>}
        {children}
      </div>
    </div>
  );
}
