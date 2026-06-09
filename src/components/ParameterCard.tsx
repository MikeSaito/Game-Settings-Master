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
        <div className="flex items-start gap-3">
          <div className="min-w-0 flex-1">
            <div
              className={cn(
                "font-semibold leading-snug",
                isOff ? "text-muted" : "text-[var(--color-text)]",
              )}
            >
              {param.title}
            </div>
            <div className="mt-1 truncate font-mono text-xs text-code">{param.key}</div>
          </div>

          <div className="flex shrink-0 items-start gap-2">
            <div className="flex flex-col items-end gap-1">
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
            {engineToggleable && onEngineToggle && (
              <Toggle
                checked={engineEnabled ?? false}
                onChange={onEngineToggle}
              />
            )}
          </div>
        </div>

        {dependencyLabel && canEdit && (
          <p className="mt-2 text-xs text-accent/90">{dependencyLabel}</p>
        )}

        {lockedReason && (
          <p className="mt-2 text-xs text-muted">{lockedReason}</p>
        )}

        {engineToggleable && !lockedReason && (
          <p className="mt-2 text-xs text-muted">
            {isOff
              ? "Выкл — строки нет в Engine.ini. Включите и нажмите «Применить», чтобы добавить."
              : "Вкл — строка есть в Engine.ini. Выключите, чтобы удалить при применении."}
          </p>
        )}

        <div
          className={cn(
            "mt-3 rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-2.5",
            isOff && "opacity-80",
          )}
        >
          <p className="text-sm leading-relaxed text-body">{param.description}</p>
          <p className="mt-2.5 text-sm text-body">
            <span className="font-medium text-[var(--color-text-secondary)]">На практике: </span>
            {param.impact}
          </p>
          {param.value_hint && (
            <p className="mt-2 text-sm text-body">
              <span className="font-medium text-muted">Рекомендуемые значения: </span>
              {param.value_hint}
            </p>
          )}
          {param.in_game_label && (
            <p className="mt-2 text-sm text-muted">
              В меню игры ищите: <span className="text-[var(--color-text-secondary)]">{param.in_game_label}</span>
            </p>
          )}
        </div>

        <div className="mt-4 flex flex-wrap items-center gap-3">
          {canEdit ? (
            param.value_type === "bool" ? (
              <select
                value={param.value}
                onChange={(e) => onChange(e.target.value)}
                className={inputClass}
                onClick={(e) => e.stopPropagation()}
              >
                <option value="True">True</option>
                <option value="False">False</option>
              </select>
            ) : param.value_type === "enum" ? (
              <select
                value={param.value}
                onChange={(e) => onChange(e.target.value)}
                className={inputClass}
                onClick={(e) => e.stopPropagation()}
              >
                <option value="On">On</option>
                <option value="Off">Off</option>
              </select>
            ) : param.value_type === "int" || param.value_type === "float" ? (
              <>
                <input
                  type="number"
                  min={param.min ?? undefined}
                  max={param.max ?? undefined}
                  step={param.value_type === "float" ? "any" : "1"}
                  value={param.value}
                  onChange={(e) => onChange(e.target.value)}
                  className={cn(inputClass, "min-w-[120px] flex-1")}
                  onClick={(e) => e.stopPropagation()}
                />
                {qualityName && <Badge tone="accent">{qualityName}</Badge>}
              </>
            ) : selectOptions?.length ? (
              <select
                value={
                  selectOptions.includes(param.value)
                    ? param.value
                    : selectOptions[0]
                }
                onChange={(e) => onChange(e.target.value)}
                className={inputClass}
                onClick={(e) => e.stopPropagation()}
              >
                {selectOptions.map((opt) => (
                  <option key={opt} value={opt}>
                    {opt}
                  </option>
                ))}
              </select>
            ) : (
              <input
                value={param.value}
                onChange={(e) => onChange(e.target.value)}
                className={cn(inputClass, "min-w-[120px] flex-1")}
                onClick={(e) => e.stopPropagation()}
              />
            )
          ) : param.value_type === "opaque" ? (
            <code className="block max-h-24 overflow-hidden rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-hover)] px-3 py-1.5 font-mono text-xs text-muted break-all">
              {param.value.length > 80 ? `${param.value.slice(0, 80)}…` : param.value}
            </code>
          ) : (
            <code
              className={cn(
                "rounded-lg border px-3 py-1.5 font-mono text-sm",
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
          )}
          {sentinel && (
            <span className="text-sm text-muted">
              В ini стоит −1: игра подставляет своё значение (часто как «по умолчанию» в меню).
              Задайте 0–4 или другое число, чтобы зафиксировать вручную.
            </span>
          )}
          {range && (
            <span className="text-sm text-muted">
              Диапазон: {range} · примерный (может отличаться)
            </span>
          )}
        </div>

        <p className="mt-3 font-mono text-xs text-faint">
          {param.file} → [{param.section}]
          {param.file === ENGINE_INI && !isOff && !param.present_in_ini && (
            <span className="text-[var(--color-accent)]"> · новый ключ</span>
          )}
        </p>
      </div>
    </Card>
  );
});
