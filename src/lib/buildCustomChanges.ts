import type { GameParameter, GpuCapabilities } from "./types";
import { isParamVisible } from "./gpuCompat";
import {
  defaultValueFor,
  isEngineEnabled,
  isEngineToggleable,
  shouldIncludeInApply,
} from "./engineParams";
import { reconcileAllParams } from "./paramDependencies";
import { paramValuesEqual } from "./paramValueEqual";

function sectionKeyFor(p: GameParameter): string {
  if (p.file === "UserConfigSelections") return p.section;
  if (p.file === "boot.config") return "";
  return p.section.startsWith("[") ? p.section : `[${p.section}]`;
}

function normalizeSectionKey(section: string): string {
  let s = section.trim();
  if (s.startsWith("[") && s.endsWith("]")) {
    s = s.slice(1, -1);
  }
  return s.toLowerCase();
}

/** Collapses sections that differ only by case (typical SN2 GameUserSettings). */
function setIniValue(
  files: Record<string, Record<string, Record<string, string>>>,
  file: string,
  sectionKey: string,
  key: string,
  value: string,
): void {
  if (!files[file]) files[file] = {};
  const fileSections = files[file];
  const normalized = normalizeSectionKey(sectionKey);
  const existingKey = Object.keys(fileSections).find(
    (k) => normalizeSectionKey(k) === normalized,
  );
  const targetKey = existingKey ?? sectionKey;
  if (!fileSections[targetKey]) fileSections[targetKey] = {};
  fileSections[targetKey][key] = value;
}

function catalogParamId(p: Pick<GameParameter, "file" | "section" | "key">): string {
  return `${p.file.toLowerCase()}|${p.section.toLowerCase()}|${p.key.toLowerCase()}`;
}

export function buildCustomChanges(
  params: GameParameter[],
  parameters: GameParameter[],
  gpu: GpuCapabilities | undefined,
  engineEnabled: Set<string>,
  editableCategories: Set<string>,
): {
  files: Record<string, Record<string, Record<string, string>>>;
  removals: Record<string, Record<string, string[]>>;
} {
  const files: Record<string, Record<string, Record<string, string>>> = {};
  const removals: Record<string, Record<string, string[]>> = {};
  const reconciled = reconcileAllParams(params, gpu);
  const baselineByCatalogId = new Map(
    parameters.map((p) => [catalogParamId(p), p]),
  );

  for (const p of reconciled) {
    if (!isParamVisible(p, gpu)) continue;
    if (!shouldIncludeInApply(p, engineEnabled)) continue;
    if (!editableCategories.has(p.category)) continue;

    const baseline = baselineByCatalogId.get(catalogParamId(p));
    const value =
      p.value.trim() || (isEngineToggleable(p) ? defaultValueFor(p) : "");
    if (!value) continue;

    if (baseline && paramValuesEqual(value, baseline.value)) continue;

    setIniValue(files, p.file, sectionKeyFor(p), p.key, value);
  }

  for (const p of parameters) {
    if (p.file !== "Engine.ini" || !p.present_in_ini || !isEngineToggleable(p)) {
      continue;
    }
    if (isEngineEnabled(p, engineEnabled)) continue;
    const sectionKey = sectionKeyFor(p);
    if (!removals[p.file]) removals[p.file] = {};
    if (!removals[p.file][sectionKey]) removals[p.file][sectionKey] = [];
    if (!removals[p.file][sectionKey].includes(p.key)) {
      removals[p.file][sectionKey].push(p.key);
    }
  }

  return { files, removals };
}
