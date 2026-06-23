import i18n from "@/i18n";

/** Values meaning "leave as the game intended" in UE ini. */
export function isUeSentinelValue(value: string): boolean {
  const v = value.trim();
  return v === "-1" || v === "-1.0" || v === "-1.000000";
}

export function formatParamDisplayValue(value: string): string {
  const v = value.trim();
  if (!v) return "—";
  if (isUeSentinelValue(v)) {
    return i18n.t("common:paramSentinel");
  }
  return v;
}

/** Clamp numeric param edits to catalog min/max when defined. */
export function clampParamValue(
  value: string,
  param: { min?: string | null; max?: string | null; value_type?: string },
): string {
  if (isUeSentinelValue(value)) return value;
  const min = param.min != null ? Number(param.min) : NaN;
  const max = param.max != null ? Number(param.max) : NaN;
  if (!Number.isFinite(min) || !Number.isFinite(max) || max <= min) {
    return value;
  }
  const n = Number(value);
  if (!Number.isFinite(n)) return value;
  const clamped =
    param.value_type === "int"
      ? Math.round(Math.min(Math.max(n, min), max))
      : Math.min(Math.max(n, min), max);
  if (param.value_type === "int") return String(clamped);
  const decimals = (String(param.min).split(".")[1] ?? "").length;
  return decimals > 0 ? clamped.toFixed(decimals) : String(clamped);
}

export function readOnlyReason(
  param: { editable: boolean; key: string },
  engineToggleable: boolean,
  engineEnabled: boolean,
): string | null {
  if (!param.editable) {
    return i18n.t("common:paramReadOnly");
  }
  if (engineToggleable && !engineEnabled) {
    return i18n.t("common:paramDisabled");
  }
  return null;
}
