import i18n from "../i18n";

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
