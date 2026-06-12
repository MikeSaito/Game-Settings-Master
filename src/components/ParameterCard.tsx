import { memo } from "react";
import type { GameParameter } from "../lib/types";
import { cn } from "../lib/cn";
import {
  formatParamDisplayValue,
  isUeSentinelValue,
  readOnlyReason,
} from "../lib/paramValue";
import { Badge } from "./ui/Badge";
import { Card } from "./ui/Card";
import { Toggle } from "./ui/Toggle";
import { ENGINE_INI } from "../lib/engineParams";

interface Props {
  param: GameParameter;
  selected?: boolean;
  onSelect?: () => void;
  editable?: boolean;
  onChange?: (value: string) => void;
  /** Тоггл «в Engine.ini» — только для CVars Engine.ini */
  engineEnabled?: boolean;
  onEngineToggle?: (enabled: boolean) => void;
  engineToggleable?: boolean;
  /** Ограниченный список значений (например, без DLSS на AMD). */
  selectOptions?: string[];
  /** Параметр синхронизируется с другим — подсказка в UI. */
  dependencyLabel?: string;
}

const QUALITY_LABELS = ["Low", "Medium", "High", "Epic", "Cinematic"];

function qualityLabel(value: string, max: number): string | null {
  if (isUeSentinelValue(value)) return "По умолчанию";
  const n = Number(value);
  if (!Number.isInteger(n) || n < 0 || n > max) return null;
  if (n <= 4 && QUALITY_LABELS[n]) return QUALITY_LABELS[n];
  if (n > 4) return `Custom @${n}`;
  return null;
}

const inputClass = "input-field rounded-lg px-3 py-2 font-mono text-sm";

type ControlKind = "toggle" | "select" | "slider" | "number" | "text";

/** Максимальный размах целочисленного слайдера — выше уходим в поле ввода. */
const MAX_INT_SLIDER_SPAN = 32;

/** Пара значений вкл/выкл для тоггла в зависимости от типа параметра. */
function toggleStates(param: GameParameter): { on: string; off: string } {
  if (param.value_type === "bool") return { on: "True", off: "False" };
  if (param.value_type === "enum") return { on: "On", off: "Off" };
  return { on: "1", off: "0" };
}

/** Шаг слайдера: явный из каталога либо «красивый» по размаху диапазона. */
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

/** Числовое сравнение значений (чтобы "100" и "100.000000" считались равными). */
function sameValue(a: string, b: string): boolean {
  const na = Number(a);
  const nb = Number(b);
  if (Number.isFinite(na) && Number.isFinite(nb)) return na === nb;
  return a.trim() === b.trim();
}

/** Определяет тип интерактивного контрола: явный ui_control из каталога или вывод по типу/диапазону. */
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

  // bool / enum — это всегда вкл/выкл, удобнее тогглом.
  if (param.value_type === "bool" || param.value_type === "enum") {
    return sentinel ? "number" : "toggle";
  }

  if (param.value_type === "int" || param.value_type === "float") {
    if (sentinel) return "number";
    const min = Number(param.min);
    const max = Number(param.max);
    if (Number.isFinite(min) && Number.isFinite(max) && max > min) {
      if (param.value_type === "int" && min === 0 && max === 1) return "toggle";
      // Слишком широкий целочисленный диапазон (разрешение теней, FPS-лимит и т.п.)
      // неудобен слайдером — отдаём числовое поле.
      if (param.value_type === "int" && max - min > MAX_INT_SLIDER_SPAN) {
        return "number";
      }
      return "slider";
    }
    return "number";
  }

  return "text";
}

export const ParameterCard = memo(function ParameterCard({
  param,
  selected,
  onSelect,
  editable,
  onChange,
  engineEnabled,
  onEngineToggle,
  engineToggleable,
  selectOptions,
  dependencyLabel,
}: Props) {
  const isOff = engineToggleable && engineEnabled === false;
  const canEdit = editable && param.editable && onChange && !isOff;
  const lockedReason = readOnlyReason(
    param,
    !!engineToggleable,
    engineEnabled ?? false,
  );
  const sentinel = isUeSentinelValue(param.value);

  const range =
    param.min != null && param.max != null && !sentinel
      ? `${param.min} – ${param.max}`
      : null;

  const maxNum = param.max != null ? Number(param.max) : NaN;

  const qualityName =
    param.key.startsWith("sg.") &&
    param.key !== "sg.ResolutionQuality" &&
    Number.isFinite(maxNum)
      ? qualityLabel(param.value, maxNum)
      : null;

  const controlKind = canEdit ? resolveControlKind(param, selectOptions, sentinel) : "text";

  const resetTarget =
    canEdit &&
    param.recommended != null &&
    param.recommended !== "" &&
    !sameValue(param.recommended, param.value)
      ? { value: param.recommended, label: "рекоменд." }
      : canEdit &&
          param.default_value != null &&
          param.default_value !== "" &&
          !sameValue(param.default_value, param.value)
        ? { value: param.default_value, label: "по умолч." }
        : null;

  const valueControl = canEdit ? (
    controlKind === "toggle" ? (
      (() => {
        const states = toggleStates(param);
        const v = param.value.trim().toLowerCase();
        const checked =
          v === states.on.toLowerCase() || v === "true" || v === "on" || v === "1";
        return (
          <div
            className="flex items-center gap-2"
            onClick={(e) => e.stopPropagation()}
          >
            <Toggle
              checked={checked}
              onChange={(next) => onChange(next ? states.on : states.off)}
            />
            <span className="font-mono text-xs text-muted">{param.value}</span>
          </div>
        );
      })()
    ) : controlKind === "select" ? (
      (() => {
        const opts = param.options?.length
          ? param.options
          : selectOptions?.length
            ? selectOptions.map((o) => ({ value: o, label: o }))
            : param.value_type === "bool"
              ? [
                  { value: "True", label: "True" },
                  { value: "False", label: "False" },
                ]
              : [
                  { value: "On", label: "On" },
                  { value: "Off", label: "Off" },
                ];
        const known = opts.some((o) => o.value === param.value);
        return (
          <select
            value={known ? param.value : opts[0]?.value}
            onChange={(e) => onChange(e.target.value)}
            className={cn(inputClass, "w-full")}
            onClick={(e) => e.stopPropagation()}
          >
            {!known && (
              <option value={param.value}>{param.value} (текущее)</option>
            )}
            {opts.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>
        );
      })()
    ) : controlKind === "slider" ? (
      (() => {
        const min = Number(param.min);
        const max = Number(param.max);
        const step = resolveStep(param, min, max);
        const current = Number(param.value);
        const safeCurrent = Number.isFinite(current)
          ? Math.min(Math.max(current, min), max)
          : min;
        return (
          <div
            className="flex flex-col gap-1"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="flex items-center gap-3">
              <input
                type="range"
                min={min}
                max={max}
                step={step}
                value={safeCurrent}
                onChange={(e) => onChange(e.target.value)}
                className="h-2 flex-1 cursor-pointer accent-[var(--color-accent)]"
              />
              <input
                type="number"
                min={param.min ?? undefined}
                max={param.max ?? undefined}
                step={step}
                value={param.value}
                onChange={(e) => onChange(e.target.value)}
                className={cn(inputClass, "w-20 shrink-0")}
              />
            </div>
            <div className="flex justify-between font-mono text-[10px] text-faint">
              <span>{param.min}</span>
              <span>{param.max}</span>
            </div>
          </div>
        );
      })()
    ) : controlKind === "number" ? (
      <input
        type="number"
        min={param.min ?? undefined}
        max={param.max ?? undefined}
        step={param.step ?? (param.value_type === "float" ? "any" : "1")}
        value={param.value}
        onChange={(e) => onChange(e.target.value)}
        className={cn(inputClass, "w-full")}
        onClick={(e) => e.stopPropagation()}
      />
    ) : (
      <input
        value={param.value}
        onChange={(e) => onChange(e.target.value)}
        className={cn(inputClass, "w-full")}
        onClick={(e) => e.stopPropagation()}
      />
    )
  ) : param.value_type === "opaque" ? (
    <code className="block max-h-24 w-full overflow-hidden rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-hover)] px-3 py-1.5 font-mono text-xs text-muted break-all">
      {param.value.length > 80 ? `${param.value.slice(0, 80)}…` : param.value}
    </code>
  ) : (
    <code
      className={cn(
        "block w-full rounded-lg border px-3 py-1.5 font-mono text-sm",
        isOff
          ? "border-[var(--color-border)] bg-[var(--color-bg-hover)] text-muted"
          : "border-[#2d5a40] bg-[#1a2e24] text-[#8fd9a8]",
      )}
    >
      {isOff
        ? formatParamDisplayValue(param.default_value || param.value || "")
        : formatParamDisplayValue(param.value)}
      {!isOff && qualityName && !sentinel ? ` · ${qualityName}` : ""}
    </code>
  );

  return (
    <Card
      selected={selected}
      className={cn(
        onSelect && "cursor-pointer",
        isOff && "border-dashed opacity-55 saturate-50",
      )}
      padding="md"
    >
      <div
        onClick={onSelect}
        onKeyDown={onSelect ? (e) => e.key === "Enter" && onSelect() : undefined}
        role={onSelect ? "button" : undefined}
        tabIndex={onSelect ? 0 : undefined}
      >
        {/* Одна строка: слева параметр и описание, справа — поле значения на всю ширину колонки. */}
        <div className="flex flex-col gap-3 md:flex-row md:items-start md:gap-6">
          <div className="min-w-0 flex-1">
            <div className="flex flex-wrap items-center gap-2">
              <span
                className={cn(
                  "font-semibold leading-snug",
                  isOff ? "text-muted" : "text-[var(--color-text)]",
                )}
              >
                {param.title}
              </span>
              <Badge tone="default" className="!px-1.5 !py-0 !text-[10px]">
                {param.category}
              </Badge>
              {engineToggleable && (
                <Badge
                  tone={isOff ? "default" : "success"}
                  className="!px-1.5 !py-0 !text-[10px]"
                >
                  {isOff ? "Нет в ini" : "В ini"}
                </Badge>
              )}
              {!param.known && (
                <Badge tone="warning" className="!px-1.5 !py-0 !text-[10px]">
                  Без описания
                </Badge>
              )}
              {!param.editable && (
                <Badge tone="default" className="!px-1.5 !py-0 !text-[10px]">
                  Служебный
                </Badge>
              )}
            </div>
            <div className="mt-1 truncate font-mono text-xs text-code">{param.key}</div>

            <p className="mt-2 text-sm leading-relaxed text-body">{param.description}</p>
            <p className="mt-1.5 text-sm text-body">
              <span className="font-medium text-[var(--color-text-secondary)]">На практике: </span>
              {param.impact}
            </p>
            {param.value_hint && (
              <p className="mt-1.5 text-sm text-body">
                <span className="font-medium text-muted">Рекомендуемые значения: </span>
                {param.value_hint}
              </p>
            )}
            {param.in_game_label && (
              <p className="mt-1.5 text-sm text-muted">
                В меню игры ищите:{" "}
                <span className="text-[var(--color-text-secondary)]">{param.in_game_label}</span>
              </p>
            )}
            {dependencyLabel && canEdit && (
              <p className="mt-2 text-xs text-accent/90">{dependencyLabel}</p>
            )}
            {lockedReason && <p className="mt-2 text-xs text-muted">{lockedReason}</p>}
            {engineToggleable && !lockedReason && (
              <p className="mt-2 text-xs text-muted">
                {isOff
                  ? "Выкл — строки нет в Engine.ini. Включите и нажмите «Применить», чтобы добавить."
                  : "Вкл — строка есть в Engine.ini. Выключите, чтобы удалить при применении."}
              </p>
            )}
          </div>

          <div className="flex w-full shrink-0 flex-col gap-2 md:w-72">
            <div className="flex items-center gap-2">
              <div className="min-w-0 flex-1">{valueControl}</div>
              {engineToggleable && onEngineToggle && (
                <Toggle checked={engineEnabled ?? false} onChange={onEngineToggle} />
              )}
            </div>
            {resetTarget && onChange && (
              <button
                type="button"
                onClick={(e) => {
                  e.stopPropagation();
                  onChange(resetTarget.value);
                }}
                className="self-start rounded-md px-2 py-0.5 text-xs text-accent transition hover:bg-[var(--color-bg-hover)]"
              >
                ↺ {resetTarget.label}: {resetTarget.value}
              </button>
            )}
            {!isOff && qualityName && canEdit && (
              <Badge tone="accent" className="self-start">
                {qualityName}
              </Badge>
            )}
            {sentinel && (
              <span className="text-xs text-muted">
                В ini стоит −1: игра подставляет своё значение. Задайте 0–4 или другое число, чтобы
                зафиксировать вручную.
              </span>
            )}
            {range && (
              <span className="text-xs text-muted">
                Диапазон: {range} · примерный
              </span>
            )}
            <p className="font-mono text-[11px] text-faint">
              {param.file} → [{param.section}]
              {param.file === ENGINE_INI && !isOff && !param.present_in_ini && (
                <span className="text-[var(--color-accent)]"> · новый ключ</span>
              )}
            </p>
          </div>
        </div>
      </div>
    </Card>
  );
});
