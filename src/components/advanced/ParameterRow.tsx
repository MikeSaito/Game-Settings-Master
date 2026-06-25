import { memo } from "react";
import { MoreHorizontal } from "lucide-react";
import { useTranslation } from "react-i18next";
import { ENGINE_INI } from "@/lib/editor";
import { cn } from "@/lib/core";
import type { GameParameter } from "@/lib/core";
import { clampParamValue, isUeSentinelValue, readOnlyReason } from "@/lib/editor";
import { Badge } from "@/components/ds/Badge";
import { Button } from "@/components/ds/Button";
import { ParameterRowControl } from "./ParameterRowControl";
import {
  qualityLabel,
  resolveControlKind,
  resolveDisplayTitle,
  sameValue,
} from "./parameterRowUtils";

interface Props {
  param: GameParameter;
  editable?: boolean;
  onChange?: (value: string) => void;
  engineEnabled?: boolean;
  onEngineToggle?: (enabled: boolean) => void;
  engineToggleable?: boolean;
  selectOptions?: string[];
  dependencyLabel?: string;
  conflictLabel?: string;
}

export const ParameterRow = memo(function ParameterRow({
  param,
  editable,
  onChange,
  engineEnabled,
  onEngineToggle,
  engineToggleable,
  selectOptions,
  dependencyLabel,
  conflictLabel,
}: Props) {
  const { t } = useTranslation("advanced");
  const isOff = engineToggleable && engineEnabled === false;
  const canEdit = editable && param.editable && onChange && !isOff;
  const lockedReason = readOnlyReason(param, !!engineToggleable, engineEnabled ?? false);
  const displayTitle = resolveDisplayTitle(param);
  const sentinel = isUeSentinelValue(param.value);
  const maxNum = param.max != null ? Number(param.max) : NaN;
  const qualityName =
    param.key.startsWith("sg.") &&
    param.key !== "sg.ResolutionQuality" &&
    Number.isFinite(maxNum)
      ? qualityLabel(param.value, maxNum, t("qualityDefault"), t)
      : null;
  const controlKind = canEdit ? resolveControlKind(param, selectOptions, sentinel) : "text";
  const changed =
    (param.recommended != null && param.recommended !== "" && !sameValue(param.recommended, param.value)) ||
    (engineToggleable && engineEnabled !== param.present_in_ini);
  const resetTarget =
    canEdit &&
    param.recommended != null &&
    param.recommended !== "" &&
    !sameValue(param.recommended, param.value)
      ? { value: param.recommended, label: t("reset.recommended") }
      : canEdit &&
          param.default_value != null &&
          param.default_value !== "" &&
          !sameValue(param.default_value, param.value)
        ? { value: param.default_value, label: t("reset.default") }
        : null;

  const emitChange = (next: string) => {
    onChange?.(clampParamValue(next, param));
  };

  return (
    <div
      className={cn(
        "group grid min-h-[58px] grid-cols-[minmax(5.5rem,6.5rem)_minmax(0,1fr)_minmax(180px,280px)_24px] items-start gap-3 border-b border-[var(--color-border)] bg-[var(--color-bg-soft)] px-3 py-2 transition hover:bg-[var(--color-surface)]",
        changed && "border-l-[3px] border-l-[var(--color-accent)] bg-[var(--color-accent-soft)]",
        isOff && "opacity-55 saturate-50",
      )}
      data-testid="parameter-row"
    >
      <div className="flex shrink-0 justify-center self-start px-0.5 pt-1">
        {engineToggleable && onEngineToggle ? (
          engineEnabled ? (
            <Button
              type="button"
              size="sm"
              variant="ghost"
              onClick={() => onEngineToggle(false)}
              title={t("engineToggle.on")}
              aria-label={t("engineIni.removeFromIni")}
              className="!h-auto min-h-8 !px-1.5 !py-1 text-[10px] leading-tight whitespace-normal text-center"
            >
              {t("engineIni.removeFromIni")}
            </Button>
          ) : (
            <Button
              type="button"
              size="sm"
              variant="secondary"
              onClick={() => onEngineToggle(true)}
              title={t("engineToggle.off")}
              aria-label={t("engineIni.addToIni")}
              className="!h-auto min-h-8 !px-1.5 !py-1 text-[10px] leading-tight whitespace-normal text-center"
            >
              {t("engineIni.addToIni")}
            </Button>
          )
        ) : (
          <span className="h-1.5 w-1.5 rounded-full bg-[var(--color-border-strong)]" />
        )}
      </div>

      <div className="min-w-0">
        <div className="flex min-w-0 items-center gap-2">
          <span className="truncate font-medium text-[var(--color-text)]">{displayTitle}</span>
          {qualityName && <Badge tone="accent" title={param.tier_hint ?? undefined}>{qualityName}</Badge>}
          {!param.known && <Badge tone="warning">{t("undocumented")}</Badge>}
          {!param.editable && <Badge tone="neutral">{t("service")}</Badge>}
          {dependencyLabel && <Badge tone="info">{dependencyLabel}</Badge>}
          {conflictLabel && <Badge tone="warning">{conflictLabel}</Badge>}
        </div>
        <div className="mt-1 flex min-w-0 flex-wrap items-center gap-x-2 gap-y-1 text-[11px] text-[var(--color-text-muted)]">
          <code className="truncate font-mono text-[var(--color-accent-hover)]">{param.key}</code>
          <span>{t(`category.${param.category}`, { defaultValue: param.category })}</span>
          <span>{param.file}</span>
          {param.file === ENGINE_INI && !isOff && !param.present_in_ini && (
            <span className="text-[var(--color-accent)]">{t("newKey")}</span>
          )}
        </div>
        <div className="hidden pt-2 text-xs leading-relaxed text-[var(--color-text-secondary)] group-hover:block">
          <p>{param.description}</p>
          <p className="mt-1">
            <span className="font-medium">{t("inPractice")}</span>
            {param.impact}
          </p>
          {param.value_hint && <p className="mt-1">{param.value_hint}</p>}
          {param.tier_hint && <p className="mt-1 text-[var(--color-text-muted)]">{param.tier_hint}</p>}
          {lockedReason && <p className="mt-1 text-[var(--color-warning)]">{lockedReason}</p>}
        </div>
      </div>

      <div className="flex min-w-0 flex-col items-end gap-1 self-start pt-0.5">
        <ParameterRowControl
          param={param}
          controlKind={controlKind}
          canEdit={!!canEdit}
          isOff={!!isOff}
          qualityName={qualityName}
          sentinel={sentinel}
          selectOptions={selectOptions}
          onChange={emitChange}
        />
        {param.key === "sg.ResolutionQuality" && (
          <span className="text-[11px] text-[var(--color-text-muted)]">
            {Number.isFinite(Number(param.value)) ? `${param.value}%` : param.value}
          </span>
        )}
        {resetTarget && (
          <button
            type="button"
            onClick={() => onChange?.(resetTarget.value)}
            className="text-[11px] text-[var(--color-accent-hover)] hover:underline"
          >
            {t("resetTo", { label: resetTarget.label, value: resetTarget.value })}
          </button>
        )}
      </div>

      <button
        type="button"
        className="grid h-7 w-7 shrink-0 place-items-center self-start rounded-[var(--radius-control)] text-[var(--color-text-faint)] opacity-0 transition hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text-secondary)] group-hover:opacity-100"
        title={`${param.file} -> [${param.section}]`}
      >
        <MoreHorizontal size={16} />
      </button>
    </div>
  );
});
