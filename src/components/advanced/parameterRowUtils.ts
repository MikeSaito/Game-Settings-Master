import type { TFunction } from "i18next";
import type { GameParameter } from "@/lib/core";
import { humanizeCvarKey, isUeSentinelValue } from "@/lib/editor";

export type ControlKind = "toggle" | "select" | "slider" | "number" | "text";

export const QUALITY_KEYS = ["0", "1", "2", "3", "4"] as const;

export const MAX_INT_SLIDER_SPAN = 32;

export const parameterInputClass =
  "h-8 rounded-[var(--radius-control)] border border-[var(--color-border)] bg-[var(--color-bg-soft)] px-2 font-mono text-xs text-[var(--color-text)] focus:border-[var(--color-accent)]";

export function optionLabel(param: GameParameter, value: string, t: TFunction<"advanced">): string {
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

export function qualityLabel(
  value: string,
  max: number,
  defaultLabel: string,
  t: TFunction<"advanced">,
): string | null {
  if (isUeSentinelValue(value)) return defaultLabel;
  const n = Number(value);
  if (!Number.isInteger(n) || n < 0 || n > max) return null;
  if (n <= 4 && QUALITY_KEYS[n as 0 | 1 | 2 | 3 | 4]) {
    return t(`qualityLevel.${QUALITY_KEYS[n as 0 | 1 | 2 | 3 | 4]}`, {
      defaultValue: QUALITY_KEYS[n as 0 | 1 | 2 | 3 | 4],
    });
  }
  if (n > 4) return t("qualityLevel.custom", { value: n });
  return null;
}

export function toggleStates(param: GameParameter): { on: string; off: string } {
  if (param.value_type === "bool") return { on: "True", off: "False" };
  if (param.value_type === "enum") return { on: "On", off: "Off" };
  return { on: "1", off: "0" };
}

export function fullscreenOptions(
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

export function resolveStep(param: GameParameter, min: number, max: number): number {
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

export function sameValue(a: string, b: string): boolean {
  const na = Number(a);
  const nb = Number(b);
  if (Number.isFinite(na) && Number.isFinite(nb)) return na === nb;
  return a.trim() === b.trim();
}

export function isPoorTitle(title: string, key: string): boolean {
  const t = title.trim();
  if (!t || t === key) return true;
  if (t.replace(/[.\s·]/g, "").toLowerCase() === key.replace(/[.\s]/g, "").toLowerCase()) {
    return true;
  }
  return false;
}

export function resolveDisplayTitle(param: GameParameter, t: TFunction<"advanced">): string {
  if (isPoorTitle(param.title, param.key)) {
    return humanizeCvarKey(param.key, t);
  }
  return param.title;
}

export function resolveControlKind(
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
