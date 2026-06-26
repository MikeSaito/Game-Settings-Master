import { useTranslation } from "react-i18next";
import { cn } from "@/lib/core";
import type { GameParameter } from "@/lib/core";
import { formatParamDisplayValue } from "@/lib/editor";
import { Select } from "@/components/ds/Field";
import { Switch } from "@/components/ds/Switch";
import type { ControlKind } from "./parameterRowUtils";
import {
  fullscreenOptions,
  optionLabel,
  parameterInputClass,
  resolveStep,
  toggleStates,
} from "./parameterRowUtils";

interface Props {
  param: GameParameter;
  controlKind: ControlKind;
  canEdit: boolean;
  isOff: boolean;
  qualityName: string | null;
  sentinel: boolean;
  selectOptions?: string[];
  onChange: (value: string) => void;
}

export function ParameterRowControl({
  param,
  controlKind,
  canEdit,
  isOff,
  qualityName,
  sentinel,
  selectOptions,
  onChange,
}: Props) {
  const { t } = useTranslation("advanced");

  if (!canEdit) {
    return (
      <code className="block max-w-[220px] truncate rounded-[var(--radius-control)] border border-[var(--color-border)] bg-[var(--color-bg-soft)] px-2 py-1.5 font-mono text-xs text-[var(--color-text-secondary)]">
        {isOff ? formatParamDisplayValue(param.default_value || param.value || "") : formatParamDisplayValue(param.value)}
        {!isOff && qualityName && !sentinel ? ` · ${qualityName}` : ""}
      </code>
    );
  }

  if (controlKind === "toggle") {
    const states = toggleStates(param);
    const v = param.value.trim().toLowerCase();
    const checked = v === states.on.toLowerCase() || v === "true" || v === "on" || v === "1";
    return (
      <div className="flex items-center justify-end gap-2">
        <Switch checked={checked} onChange={(next) => onChange(next ? states.on : states.off)} />
        <span className="w-10 truncate font-mono text-xs text-[var(--color-text-muted)]">{param.value}</span>
      </div>
    );
  }

  if (controlKind === "select") {
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
        onChange={(event) => onChange(event.target.value)}
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
  }

  if (controlKind === "slider") {
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
          onChange={(event) => onChange(event.target.value)}
          className="w-28 accent-[var(--color-accent)]"
        />
        <input
          type="number"
          min={param.min ?? undefined}
          max={param.max ?? undefined}
          step={step}
          value={param.value}
          onChange={(event) => onChange(event.target.value)}
          className={cn(parameterInputClass, "w-20")}
        />
      </div>
    );
  }

  return (
    <input
      type={controlKind === "number" ? "number" : "text"}
      min={param.min ?? undefined}
      max={param.max ?? undefined}
      step={param.step ?? (param.value_type === "float" ? "any" : "1")}
      value={param.value}
      onChange={(event) => onChange(event.target.value)}
      className={cn(parameterInputClass, "w-full max-w-[220px]")}
    />
  );
}
