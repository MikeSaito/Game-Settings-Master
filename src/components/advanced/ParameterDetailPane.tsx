import { useTranslation } from "react-i18next";
import type { GameParameter } from "@/lib/core";
import { readOnlyReason } from "@/lib/editor";

interface Props {
  param: GameParameter | null;
  engineToggleable?: boolean;
  engineEnabled?: boolean;
  onPointerEnter?: () => void;
  onPointerLeave?: () => void;
}

export function ParameterDetailPane({
  param,
  engineToggleable = false,
  engineEnabled = true,
  onPointerEnter,
  onPointerLeave,
}: Props) {
  const { t } = useTranslation("advanced");

  const shellClass = "min-w-0";

  if (!param) return null;

  const lockedReason = readOnlyReason(param, engineToggleable, engineEnabled);

  return (
    <div
      className={shellClass}
      data-testid="parameter-list-details"
      onMouseEnter={onPointerEnter}
      onMouseLeave={onPointerLeave}
    >
      <code className="mb-4 block break-all text-xs text-[var(--color-text-muted)]">
        {param.file} · [{param.section}]
      </code>
      <div className="text-sm leading-relaxed text-[var(--color-text-secondary)]">
        <p>{param.description}</p>
        <p className="mt-2">
          <span className="font-medium">{t("inPractice")}</span>
          {param.impact}
        </p>
        {param.value_hint && <p className="mt-2">{param.value_hint}</p>}
        {param.tier_hint && (
          <p className="mt-2 text-[var(--color-text-muted)]">
            {param.tier_hint}
          </p>
        )}
        {lockedReason && (
          <p className="mt-2 text-[var(--color-warning)]">{lockedReason}</p>
        )}
      </div>
    </div>
  );
}
