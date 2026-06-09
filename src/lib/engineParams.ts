import type { GameParameter } from "./types";

export const ENGINE_INI = "Engine.ini";

export const ENGINE_CATEGORIES = new Set([
  "Rendering",
  "Shadows",
  "Textures",
  "PostProcess",
]);

/** Уникальный id для состояния тоггла (ключ внутри Engine.ini). */
export function engineParamId(p: Pick<GameParameter, "file" | "key">): string {
  return `${p.file}::${p.key}`;
}

export function paramId(p: Pick<GameParameter, "file" | "section" | "key">): string {
  return `${p.file}|${p.section}|${p.key}`;
}

/** Параметры Engine.ini с тогглом вкл/выкл (редактируемые, не opaque). */
export function isEngineToggleable(p: GameParameter): boolean {
  return (
    p.file === ENGINE_INI &&
    p.known &&
    p.editable &&
    p.value_type !== "opaque"
  );
}

export function initialEngineEnabledKeys(parameters: GameParameter[]): Set<string> {
  const keys = new Set<string>();
  for (const p of parameters) {
    if (isEngineToggleable(p) && p.present_in_ini) {
      keys.add(engineParamId(p));
    }
  }
  return keys;
}

export function isEngineEnabled(
  p: GameParameter,
  enabled: Set<string>,
): boolean {
  if (!isEngineToggleable(p)) return true;
  return enabled.has(engineParamId(p));
}

/** Параметр попадает в files при «Применить» / «Сохранить пресет». */
export function shouldIncludeInApply(
  p: GameParameter,
  engineEnabled: Set<string>,
): boolean {
  if (p.file === ENGINE_INI) {
    if (!isEngineToggleable(p)) return false;
    return isEngineEnabled(p, engineEnabled);
  }
  return p.known && p.editable;
}

/**
 * Категории, разрешённые для записи. Всегда объединяет данные игры
 * с базовым списком — иначе Engine.ini выпадает, если в ini только служебные ключи.
 */
export function resolveEditableCategories(
  parameters: GameParameter[],
  baseCategories: ReadonlySet<string>,
  extraCategories?: ReadonlySet<string>,
): Set<string> {
  const cats = new Set(
    parameters.filter((p) => p.editable).map((p) => p.category),
  );
  for (const c of baseCategories) cats.add(c);
  for (const c of ENGINE_CATEGORIES) cats.add(c);
  if (extraCategories) {
    for (const c of extraCategories) cats.add(c);
  }
  return cats;
}

export function defaultValueFor(p: GameParameter): string {
  if (p.default_value) return p.default_value;
  if (p.value_type === "bool") return "True";
  if (p.value_type === "float") return "1.0";
  if (p.min) return p.min;
  return "1";
}
