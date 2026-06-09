import type { ReactNode } from "react";
import { cn } from "../../lib/cn";

interface Props {
  children: ReactNode;
  className?: string;
  selected?: boolean;
  hover?: boolean;
  padding?: "sm" | "md" | "lg";
}

const paddingMap = {
  sm: "p-3",
  md: "p-4",
  lg: "p-5",
};

export function Card({
  children,
  className,
  selected,
  hover = true,
  padding = "md",
}: Props) {
  return (
    <div
      className={cn(
        "surface-card rounded-xl transition duration-150",
        paddingMap[padding],
        hover && "hover:bg-[var(--color-bg-hover)]",
        selected && "surface-card-selected",
        className,
      )}
    >
      {children}
    </div>
  );
}
