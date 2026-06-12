import { Check, Sparkles, Zap } from "lucide-react";
import { useTranslation } from "react-i18next";
import { presetAccentForReShade } from "../../lib/reshade";
import type { ReShadePresetInfo } from "../../lib/types";
import { cn } from "../../lib/cn";
import { Badge } from "../ui/Badge";

interface Props {
  preset: ReShadePresetInfo;
  selected: boolean;
  installed?: boolean;
  appliesWhenAdapted?: boolean;
  disabled?: boolean;
  onSelect: () => void;
}

const HIGHLIGHT_IDS = new Set(["clarity", "sn2-underwater-clarity"]);

export function ReShadePresetCard({
  preset,
  selected,
  installed = false,
  appliesWhenAdapted = false,
  disabled,
  onSelect,
}: Props) {
  const { t } = useTranslation("reshade");
  const accent = presetAccentForReShade(preset.id);
  const showHighlight = HIGHLIGHT_IDS.has(preset.id) || preset.author;

  return (
    <button
      type="button"
      disabled={disabled}
      onClick={onSelect}
      className={cn(
        "surface-card group relative rounded-xl p-4 text-left transition",
        selected && "surface-card-selected",
        disabled && "opacity-60",
      )}
      style={{ borderLeftWidth: 3, borderLeftColor: accent }}
    >
      <div className="flex items-start justify-between gap-2">
        {preset.author ? (
          <Badge tone="accent">
            <span className="inline-flex items-center gap-1">
              <Sparkles size={10} />
              {t("presetCard.fromAuthor")}
            </span>
          </Badge>
        ) : (
          <Badge tone="default">{preset.id}</Badge>
        )}
        <div className="flex shrink-0 items-center gap-1.5">
          {installed && (
            <Badge tone="success">
              <span className="text-[10px]">{t("presetCard.inGame")}</span>
            </Badge>
          )}
          {appliesWhenAdapted && !installed && (
            <Badge tone="accent">
              <span className="text-[10px]">{t("presetCard.willApply")}</span>
            </Badge>
          )}
          {selected && (
            <span className="flex h-6 w-6 items-center justify-center rounded-full bg-[var(--color-accent-soft)] text-accent">
              <Check size={14} />
            </span>
          )}
        </div>
      </div>
      <div className="mt-3 text-base font-semibold text-[var(--color-text)]">{preset.name}</div>
      <p className="mt-1.5 text-sm leading-relaxed text-body">{preset.description}</p>
      {showHighlight && (
        <div className="mt-3">
          <Badge tone="accent">
            <span className="inline-flex items-center gap-1">
              <Zap size={10} />
              {preset.author ? t("presetCard.forThisGame") : t("presetCard.recommended")}
            </span>
          </Badge>
        </div>
      )}
    </button>
  );
}
