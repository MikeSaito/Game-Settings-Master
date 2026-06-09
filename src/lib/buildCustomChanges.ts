import type { GameParameter, GpuCapabilities } from "./types";
import { isParamVisible } from "./gpuCompat";
import {
  defaultValueFor,
  isEngineEnabled,
  isEngineToggleable,
  shouldIncludeInApply,
} from "./engineParams";
import { reconcileAllParams } from "./paramDependencies";

function sectionKeyFor(p: GameParameter): string {
  if (p.file === "UserConfigSelections") return p.section;
  if (p.file === "boot.config") return "";
  return p.section.startsWith("[") ? p.section : `[${p.section}]`;
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

  for (const p of reconciled) {
    if (!isParamVisible(p, gpu)) continue;
    if (!shouldIncludeInApply(p, engineEnabled)) continue;
    if (!editableCategories.has(p.category)) continue;

    const sectionKey = sectionKeyFor(p);
    if (!files[p.file]) files[p.file] = {};
    if (!files[p.file][sectionKey]) files[p.file][sectionKey] = {};
    const value =
      p.value.trim() || (isEngineToggleable(p) ? defaultValueFor(p) : "");
    if (value) {
      files[p.file][sectionKey][p.key] = value;
    }
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
