/** Значения «оставь как задумала игра» в UE ini. */
export function isUeSentinelValue(value: string): boolean {
  const v = value.trim();
  return v === "-1" || v === "-1.0" || v === "-1.000000";
}

export function formatParamDisplayValue(value: string): string {
  const v = value.trim();
  if (!v) return "—";
  if (isUeSentinelValue(v)) {
    return "−1 · автоматически (по умолчанию движка)";
  }
  return v;
}

export function readOnlyReason(
  param: { editable: boolean; key: string },
  engineToggleable: boolean,
  engineEnabled: boolean,
): string | null {
  if (!param.editable) {
    return "Служебный параметр — только для чтения. Игра сама обновляет это значение; ручное изменение может сломать настройки.";
  }
  if (engineToggleable && !engineEnabled) {
    return "Параметр выключен (тоггл «Выкл»). Включите и нажмите «Применить», чтобы редактировать.";
  }
  return null;
}
