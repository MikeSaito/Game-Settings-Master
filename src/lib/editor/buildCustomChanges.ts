import type { GameParameter, GpuCapabilities } from "@/lib/core/types";
import { isParamVisible } from "@/lib/gpu/gpuCompat";
import { filterParamsByPanel, type EditorPanel } from "@/lib/routing";
import {
  defaultValueFor,
  isEngineEnabled,
  isIniMembershipToggleable,
  shouldIncludeInApply,
} from "./engineParams";
import { EMPTY_INI_SNAPSHOT } from "./iniSnapshot";
import { reconcileAllParams } from "./paramDependencies";
import { paramValuesEqual } from "./paramValueEqual";

function sectionKeyFor(p: GameParameter): string {
  if (p.file === "UserConfigSelections") return p.section;
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
  panel?: EditorPanel,
  shippedIniKeys: ReadonlySet<string> = EMPTY_INI_SNAPSHOT,
): {
  files: Record<string, Record<string, Record<string, string>>>;
  removals: Record<string, Record<string, string[]>>;
} {
  const files: Record<string, Record<string, Record<string, string>>> = {};
  const removals: Record<string, Record<string, string[]>> = {};
  const reconciled = reconcileAllParams(params, gpu);
  const writeParams = panel ? filterParamsByPanel(reconciled, panel) : reconciled;
  const baselineByCatalogId = new Map(
    parameters.map((p) => [catalogParamId(p), p]),
  );

  for (const p of writeParams) {
    const baseline = baselineByCatalogId.get(catalogParamId(p));
    const value =
      p.value.trim() || (isIniMembershipToggleable(p, shippedIniKeys) ? defaultValueFor(p) : "");
    if (!value) continue;

    const addingToIni =
      isIniMembershipToggleable(p, shippedIniKeys) &&
      isEngineEnabled(p, engineEnabled, shippedIniKeys) &&
      baseline != null &&
      !baseline.present_in_ini;
    if (baseline && paramValuesEqual(value, baseline.value) && !addingToIni) continue;
    if (!isParamVisible(p, gpu) && !baseline) continue;
    if (!shouldIncludeInApply(p, engineEnabled, shippedIniKeys)) continue;
    if (!editableCategories.has(p.category)) continue;

    setIniValue(files, p.file, sectionKeyFor(p), p.key, value);
  }

  const draftByCatalogId = new Map(
    reconciled.map((p) => [catalogParamId(p), p]),
  );

  for (const p of parameters) {
    if (!p.present_in_ini) continue;
    const draft = draftByCatalogId.get(catalogParamId(p));
    if (!draft) continue;

    const sectionKey = sectionKeyFor(p);
    const addRemoval = () => {
      if (!removals[p.file]) removals[p.file] = {};
      if (!removals[p.file][sectionKey]) removals[p.file][sectionKey] = [];
      if (!removals[p.file][sectionKey].includes(p.key)) {
        removals[p.file][sectionKey].push(p.key);
      }
    };

    if (isIniMembershipToggleable(p, shippedIniKeys)) {
      if (isEngineEnabled(p, engineEnabled, shippedIniKeys)) continue;
      addRemoval();
      continue;
    }

    if (
      p.key.toLowerCase().startsWith("r.") &&
      (p.file === "Engine.ini" ||
        p.file === "Scalability.ini" ||
        p.file === "Game.ini") &&
      !draft.present_in_ini &&
      draft.value.trim() === ""
    ) {
      addRemoval();
    }
  }

  return { files, removals };
}
