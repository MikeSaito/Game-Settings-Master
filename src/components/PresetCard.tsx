import { Check, Zap } from "lucide-react";
import type { PresetInfo } from "../lib/types";
import { cn } from "../lib/cn";
import { Badge } from "./ui/Badge";

interface Props {
  preset: PresetInfo;
  selected: boolean;
  onSelect: () => void;
}

const presetAccent: Record<string, string> = {
  "ultra-low": "#e05c5c",
  low: "#e8944a",
  medium: "#d4b84a",
  high: "#7bc96f",
  epic: "#4caf82",
  "ultra-high": "#5b8def",
  potato: "#8b6f47",
  minimum: "#c97a6a",
  ultramax: "#9b6bdf",
};

function accentForPreset(id: string): string {
  if (presetAccent[id]) return presetAccent[id];
  let hash = 0;
  for (let i = 0; i < id.length; i++) {
    hash = (hash * 31 + id.charCodeAt(i)) >>> 0;
  }
  const hue = hash % 360;
  return `hsl(${hue} 45% 55%)`;
}

const HIGHLIGHT_IDS = new Set(["ultra-high", "ultramax", "epic"]);

export function PresetCard({ preset, selected, onSelect }: Props) {
  const accent = accentForPreset(preset.id);
  const showHighlight =
    HIGHLIGHT_IDS.has(preset.id) ||
    /выше|максимум|ray tracing|RT/i.test(preset.description);

  return (
    <button
      type="button"
      onClick={onSelect}
      className={cn(
        "surface-card group relative rounded-xl p-4 text-left transition",
        selected && "surface-card-selected",
      )}
      style={{ borderLeftWidth: 3, borderLeftColor: accent }}
    >
      <div className="flex items-start justify-between gap-2">
        <Badge tone="default">{preset.id}</Badge>
        {selected && (
          <span className="flex h-6 w-6 items-center justify-center rounded-full bg-[var(--color-accent-soft)] text-accent">
            <Check size={14} />
          </span>
        )}
      </div>
      <div className="mt-3 text-base font-semibold text-[var(--color-text)]">{preset.name}</div>
      <p className="mt-1.5 text-sm leading-relaxed text-body">{preset.description}</p>
      {showHighlight && (
        <div className="mt-3">
          <Badge tone="accent">
            <span className="inline-flex items-center gap-1">
              <Zap size={10} />
              {preset.id === "ultra-high" || preset.id === "ultramax"
                ? "Максимальное качество"
                : "Рекомендуемый"}
            </span>
          </Badge>
        </div>
      )}
    </button>
  );
}
