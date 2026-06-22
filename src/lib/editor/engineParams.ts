import type { GameParameter } from "../core/types";

export const ENGINE_INI = "Engine.ini";

export const ENGINE_CATEGORIES = new Set([
  "Rendering",
  "Shadows",
  "Textures",
  "PostProcess",
]);

/** Unique id for toggle state (key inside Engine.ini). */
export function engineParamId(p: Pick<GameParameter, "file" | "key">): string {
  return `${p.file}::${p.key}`;
}

export function paramId(p: Pick<GameParameter, "file" | "section" | "key">): string {
  return `${p.file}|${p.section}|${p.key}`;
}

/** Engine.ini parameters with on/off toggle (editable, not opaque). */
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

/** Parameter is included in files on Apply / Save preset. */
export function shouldIncludeInApply(
  p: GameParameter,
  engineEnabled: Set<string>,
): boolean {
  if (p.file === ENGINE_INI) {
    if (!isEngineToggleable(p)) return false;
    return isEngineEnabled(p, engineEnabled);
  }
  return p.editable && p.value_type !== "opaque";
}

/**
 * Categories allowed for writing. Always merges game data with the base list —
 * otherwise Engine.ini is omitted when ini contains only utility keys.
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
