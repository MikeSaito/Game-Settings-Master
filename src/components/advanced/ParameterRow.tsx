import { memo } from "react";
import { Info } from "lucide-react";
import { useTranslation } from "react-i18next";
import { cn } from "@/lib/core";
import type { GameParameter } from "@/lib/core";
import { clampParamValue, isUeSentinelValue } from "@/lib/editor";
import { EMPTY_INI_SNAPSHOT, isIniShippedKey } from "@/lib/editor";
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
  warningLabel?: string;
  shippedIniKeys?: ReadonlySet<string>;
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
  warningLabel,
  shippedIniKeys = EMPTY_INI_SNAPSHOT,
}: Props) {
  const { t } = useTranslation("advanced");
  const isOff = engineToggleable && engineEnabled === false;
  const canEdit = editable && param.editable && onChange && !isOff;
  const displayTitle = resolveDisplayTitle(param);
  const sentinel = isUeSentinelValue(param.value);
  const maxNum = param.max != null ? Number(param.max) : NaN;
  const qualityName =
    param.key.startsWith("sg.") &&
    param.key !== "sg.ResolutionQuality" &&
    Number.isFinite(maxNum)
      ? qualityLabel(param.value, maxNum, t("qualityDefault"), t)
      : null;
  const controlKind = canEdit
    ? resolveControlKind(param, selectOptions, sentinel)
    : "text";
  const changed =
    (param.recommended != null &&
      param.recommended !== "" &&
      !sameValue(param.recommended, param.value)) ||
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
        "parameter-row group grid min-h-[76px] items-center gap-4 border-b border-[var(--color-border)]/60 px-5 py-4 hover:bg-[var(--color-surface-raised)]",
        engineToggleable && onEngineToggle
          ? "parameter-row-membership"
          : "parameter-row-standard",
        resetTarget && "min-h-[5.75rem]",
        changed &&
          "border-l-[3px] border-l-[var(--color-accent)] bg-[var(--color-accent-soft)]",
        isOff && "opacity-55 saturate-50",
      )}
      data-testid="parameter-row"
    >
      <div
        className={cn(
          "parameter-membership flex shrink-0 justify-center self-start pt-1",
          !(engineToggleable && onEngineToggle) && "hidden",
        )}
      >
        {engineToggleable && onEngineToggle ? (
          engineEnabled ? (
            <Button
              type="button"
              size="sm"
              variant="ghost"
              onClick={() => onEngineToggle(false)}
              title={t("engineToggle.on")}
              aria-label={t("engineIni.removeFromIni")}
              className="!h-auto min-h-9 !px-2 !py-1.5 text-xs leading-tight whitespace-normal text-center"
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
              className="!h-auto min-h-9 !px-2 !py-1.5 text-xs leading-tight whitespace-normal text-center"
            >
              {t("engineIni.addToIni")}
            </Button>
          )
        ) : (
          <span className="h-1.5 w-1.5 rounded-full bg-[var(--color-border-strong)]" />
        )}
      </div>

      <div className="min-w-0">
        <div className="flex min-w-0 flex-wrap items-center gap-2">
          <span className="break-words font-medium text-[var(--color-text)]">
            {displayTitle}
          </span>
          {qualityName && (
            <Badge tone="accent" title={param.tier_hint ?? undefined}>
              {qualityName}
            </Badge>
          )}
          {!param.known && <Badge tone="warning">{t("undocumented")}</Badge>}
          {!param.editable && <Badge tone="neutral">{t("service")}</Badge>}
          {dependencyLabel && <Badge tone="info">{dependencyLabel}</Badge>}
          {conflictLabel && <Badge tone="warning">{conflictLabel}</Badge>}
          {warningLabel && !conflictLabel && (
            <Badge tone="warning">{warningLabel}</Badge>
          )}
          {isIniShippedKey(param, shippedIniKeys) && (
            <Badge tone="success">{t("iniShipped")}</Badge>
          )}
          {param.present_in_ini &&
            !isIniShippedKey(param, shippedIniKeys) &&
            (!engineToggleable || engineEnabled) && (
              <Badge tone="success">{t("inIni")}</Badge>
            )}
          {engineToggleable && engineEnabled && !param.present_in_ini && (
            <Badge tone="accent">{t("gusIni.willAdd")}</Badge>
          )}
          {engineToggleable && !engineEnabled && (
            <Badge tone="neutral">{t("gusIni.notInIni")}</Badge>
          )}
        </div>
        <div className="mt-1 flex min-w-0 flex-wrap items-center gap-x-2 gap-y-1 text-xs text-[var(--color-text-muted)]">
          <code className="truncate font-mono text-[var(--color-accent-hover)]">
            {param.key}
          </code>
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
          <span className="text-xs text-[var(--color-text-muted)]">
            {Number.isFinite(Number(param.value))
              ? `${param.value}%`
              : param.value}
          </span>
        )}
        {resetTarget && (
          <button
            type="button"
            onClick={() => onChange?.(resetTarget.value)}
            className="max-w-full text-right text-xs leading-snug text-[var(--color-accent-hover)] hover:underline"
          >
            {t("resetTo", {
              label: resetTarget.label,
              value: resetTarget.value,
            })}
          </button>
        )}
      </div>

      <button
        type="button"
        className="material-icon-button parameter-info"
        data-parameter-details
        aria-label={t("parameterDetails", { name: displayTitle })}
        title={`${param.file} -> [${param.section}]`}
      >
        <Info size={16} />
      </button>
    </div>
  );
});
