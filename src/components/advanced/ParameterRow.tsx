import { memo } from "react";
import { MoreHorizontal } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { TFunction } from "i18next";
import { ENGINE_INI } from "../../lib/engineParams";
import { cn } from "../../lib/cn";
import type { GameParameter } from "../../lib/types";
import {
  clampParamValue,
  formatParamDisplayValue,
  isUeSentinelValue,
  readOnlyReason,
} from "../../lib/paramValue";
import { humanizeCvarKey } from "../../lib/cvarHumanize";
import { Badge } from "../ds/Badge";
import { Button } from "../ds/Button";
import { Select } from "../ds/Field";
import { Switch } from "../ds/Switch";

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

type ControlKind = "toggle" | "select" | "slider" | "number" | "text";

const QUALITY_KEYS = ["0", "1", "2", "3", "4"] as const;

function optionLabel(param: GameParameter, value: string, t: TFunction<"advanced">): string {
  if (param.key === "DLSSMode") {
    return t(`dlssMode.${value}`, { defaultValue: value });
  }
  if (param.key === "UpscalingMethod") {
    return t(`upscalingMethod.${value}`, { defaultValue: value });
  }
  if (param.key === "AntiAliasingType") {
    return t(`antiAliasingType.${value}`, { defaultValue: value });
  }
  if (["FullscreenMode", "PreferredFullscreenMode", "LastConfirmedFullscreenMode"].includes(param.key)) {
    return t(`fullscreenMode.${value}`, { defaultValue: value });
  }
  if (param.value_type === "bool") {
    const lower = value.toLowerCase();
    if (lower === "true" || lower === "false") {
      return t(`boolValue.${lower}`, { defaultValue: value });
    }
  }
  if (param.value_type === "enum" && (value === "On" || value === "Off")) {
    return t(`boolValue.${value.toLowerCase()}`, { defaultValue: value });
  }
  return value;
}

function qualityLabel(
  value: string,
  max: number,
  defaultLabel: string,
  t: TFunction<"advanced">,
): string | null {
  if (isUeSentinelValue(value)) return defaultLabel;
  const n = Number(value);
  if (!Number.isInteger(n) || n < 0 || n > max) return null;
  if (n <= 4 && QUALITY_KEYS[n]) {
    return t(`qualityLevel.${QUALITY_KEYS[n]}`, { defaultValue: QUALITY_KEYS[n] });
  }
  if (n > 4) return t("qualityLevel.custom", { value: n });
  return null;
}

const MAX_INT_SLIDER_SPAN = 32;
const inputClass =
  "h-8 rounded-[var(--radius-control)] border border-[var(--color-border)] bg-[var(--color-bg-soft)] px-2 font-mono text-xs text-[var(--color-text)] focus:border-[var(--color-accent)]";

function toggleStates(param: GameParameter): { on: string; off: string } {
  if (param.value_type === "bool") return { on: "True", off: "False" };
  if (param.value_type === "enum") return { on: "On", off: "Off" };
  return { on: "1", off: "0" };
}

function fullscreenOptions(
  param: GameParameter,
  t: TFunction<"advanced">,
): { value: string; label: string }[] | null {
  if (!["FullscreenMode", "PreferredFullscreenMode", "LastConfirmedFullscreenMode"].includes(param.key)) {
    return null;
  }
  return ["0", "1", "2"].map((value) => ({
    value,
    label: t(`fullscreenMode.${value}`),
  }));
}

function resolveStep(param: GameParameter, min: number, max: number): number {
  if (param.step) {
    const explicit = Number(param.step);
    if (Number.isFinite(explicit) && explicit > 0) return explicit;
  }
  if (param.value_type === "int") return 1;
  const span = Math.abs(max - min);
  if (!Number.isFinite(span) || span <= 0) return 0.01;
  if (span <= 0.2) return 0.01;
  if (span <= 2) return 0.05;
  if (span <= 20) return 0.1;
  return Math.max(1, Math.round(span / 100));
}

function sameValue(a: string, b: string): boolean {
  const na = Number(a);
  const nb = Number(b);
  if (Number.isFinite(na) && Number.isFinite(nb)) return na === nb;
  return a.trim() === b.trim();
}

function isPoorTitle(title: string, key: string): boolean {
  const trimmed = title.trim();
  if (!trimmed) return true;
  if (trimmed.toLowerCase() === key.toLowerCase()) return true;
  const last = key.includes(".") ? key.split(".").pop() ?? key : key;
  if (trimmed.toLowerCase() === last.toLowerCase()) return true;
  return false;
}

function resolveDisplayTitle(param: GameParameter, t: TFunction<"advanced">): string {
  if (isPoorTitle(param.title, param.key)) {
    return humanizeCvarKey(param.key, t);
  }
  return param.title;
}

function resolveControlKind(
  param: GameParameter,
  selectOptions: string[] | undefined,
  sentinel: boolean,
): ControlKind {
  if (selectOptions?.length) return "select";
  if (param.options?.length) return "select";
  const explicit = param.ui_control;
  if (explicit === "toggle") return sentinel ? "number" : "toggle";
  if (explicit === "select") return "select";
  if (explicit === "slider") return sentinel ? "number" : "slider";
  if (explicit === "stepper") return "number";
  if (explicit === "text") return "text";
  if (param.value_type === "bool" || param.value_type === "enum") return sentinel ? "number" : "toggle";
  if (param.value_type === "int" || param.value_type === "float") {
    if (sentinel) return "number";
    const min = Number(param.min);
    const max = Number(param.max);
    if (Number.isFinite(min) && Number.isFinite(max) && max > min) {
      if (param.value_type === "int" && min === 0 && max === 1) return "toggle";
      if (param.value_type === "int" && max - min > MAX_INT_SLIDER_SPAN) return "number";
      return "slider";
    }
    return "number";
  }
  return "text";
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
  const displayTitle = resolveDisplayTitle(param, t);
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

  const control = canEdit ? (
    controlKind === "toggle" ? (
      (() => {
        const states = toggleStates(param);
        const v = param.value.trim().toLowerCase();
        const checked = v === states.on.toLowerCase() || v === "true" || v === "on" || v === "1";
        return (
          <div className="flex items-center justify-end gap-2">
            <Switch checked={checked} onChange={(next) => emitChange(next ? states.on : states.off)} />
            <span className="w-10 truncate font-mono text-xs text-[var(--color-text-muted)]">{param.value}</span>
          </div>
        );
      })()
    ) : controlKind === "select" ? (
      (() => {
        const fallbackFullscreen = fullscreenOptions(param, t);
        const opts = param.options?.length
          ? param.options.map((option) => ({
              value: option.value,
              label: optionLabel(param, option.value, t),
            }))
          : fallbackFullscreen
            ? fallbackFullscreen
          : selectOptions?.length
            ? selectOptions.map((value) => ({
                value,
                label: optionLabel(param, value, t),
              }))
            : param.value_type === "bool"
              ? [
                  { value: "True", label: t("boolValue.true") },
                  { value: "False", label: t("boolValue.false") },
                ]
              : [
                  { value: "On", label: t("boolValue.on") },
                  { value: "Off", label: t("boolValue.off") },
                ];
        const known = opts.some((option) => option.value === param.value);
        return (
          <Select
            value={known ? param.value : opts[0]?.value}
            onChange={(event) => emitChange(event.target.value)}
            className="max-w-[220px] font-mono text-xs"
          >
            {!known && <option value={param.value}>{t("currentOption", { value: param.value })}</option>}
            {opts.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </Select>
        );
      })()
    ) : controlKind === "slider" ? (
      (() => {
        const min = Number(param.min);
        const max = Number(param.max);
        const step = resolveStep(param, min, max);
        const current = Number(param.value);
        const safeCurrent = Number.isFinite(current) ? Math.min(Math.max(current, min), max) : min;
        return (
          <div className="flex items-center justify-end gap-2">
            <input
              type="range"
              min={min}
              max={max}
              step={step}
              value={safeCurrent}
              onChange={(event) => emitChange(event.target.value)}
              className="w-28 accent-[var(--color-accent)]"
            />
            <input
              type="number"
              min={param.min ?? undefined}
              max={param.max ?? undefined}
              step={step}
              value={param.value}
              onChange={(event) => emitChange(event.target.value)}
              className={cn(inputClass, "w-20")}
            />
          </div>
        );
      })()
    ) : (
      <input
        type={controlKind === "number" ? "number" : "text"}
        min={param.min ?? undefined}
        max={param.max ?? undefined}
        step={param.step ?? (param.value_type === "float" ? "any" : "1")}
        value={param.value}
        onChange={(event) => emitChange(event.target.value)}
        className={cn(inputClass, "w-full max-w-[220px]")}
      />
    )
  ) : (
    <code className="block max-w-[220px] truncate rounded-[var(--radius-control)] border border-[var(--color-border)] bg-[var(--color-bg-soft)] px-2 py-1.5 font-mono text-xs text-[var(--color-text-secondary)]">
      {isOff ? formatParamDisplayValue(param.default_value || param.value || "") : formatParamDisplayValue(param.value)}
      {!isOff && qualityName && !sentinel ? ` · ${qualityName}` : ""}
    </code>
  );

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
        {control}
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
