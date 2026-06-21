import { engineParamId, isEngineEnabled, isEngineToggleable } from "./engineParams";
import type { GameParameter } from "./types";

const ENGINE_INI_FILES = new Set(["Engine.ini", "Scalability.ini", "Game.ini"]);
const R_CVAR_RE = /r\.[A-Za-z0-9_.]+/g;

/** Extra r.* prefix match when sg tier is active (manual overrides beyond tier_hint). */
const SG_R_PREFIX: Record<string, string> = {
  "sg.shadowquality": "r.shadow",
};

export function extractRelatedRCvars(tierHint: string | null | undefined): string[] {
  if (!tierHint) return [];
  const matches = tierHint.match(R_CVAR_RE) ?? [];
  return [...new Set(matches.map((m) => m.toLowerCase()))];
}

function isSgActive(param: GameParameter, pendingKeys: Set<string>): boolean {
  const key = param.key.toLowerCase();
  if (pendingKeys.has(key)) return true;
  return (
    param.file === "GameUserSettings.ini" &&
    param.present_in_ini &&
    param.value.trim() !== ""
  );
}

function isREngineOverride(
  param: GameParameter,
  pendingKeys: Set<string>,
  engineEnabled: Set<string>,
): boolean {
  const key = param.key.toLowerCase();
  if (!key.startsWith("r.")) return false;
  if (!ENGINE_INI_FILES.has(param.file)) return false;
  if (pendingKeys.has(key)) return true;
  if (param.file === "Engine.ini" && isEngineToggleable(param)) {
    return isEngineEnabled(param, engineEnabled) && param.present_in_ini;
  }
  return param.present_in_ini && param.value.trim() !== "";
}

function paramsByKey(params: GameParameter[]): Map<string, GameParameter[]> {
  const map = new Map<string, GameParameter[]>();
  for (const param of params) {
    const key = param.key.toLowerCase();
    const bucket = map.get(key);
    if (bucket) bucket.push(param);
    else map.set(key, [param]);
  }
  return map;
}

/**
 * Keys that overlap sg.* GUS tiers with active r.* engine overrides (ini or pending).
 */
export function detectSgEngineConflicts(
  params: GameParameter[],
  pendingKeys: Set<string>,
  engineEnabled: Set<string>,
): Set<string> {
  const byKey = paramsByKey(params);
  const conflicts = new Set<string>();

  for (const sg of params) {
    if (!sg.key.startsWith("sg.") || sg.file !== "GameUserSettings.ini") continue;
    if (!isSgActive(sg, pendingKeys)) continue;

    const sgKey = sg.key.toLowerCase();
    const related = new Set(extractRelatedRCvars(sg.tier_hint));
    const prefix = SG_R_PREFIX[sgKey];
    if (prefix) {
      for (const param of params) {
        if (param.key.toLowerCase().startsWith(prefix)) {
          related.add(param.key.toLowerCase());
        }
      }
    }

    if (related.size === 0) continue;

    let hit = false;
    for (const rKey of related) {
      const candidates = byKey.get(rKey) ?? [];
      for (const rParam of candidates) {
        if (isREngineOverride(rParam, pendingKeys, engineEnabled)) {
          conflicts.add(rKey);
          hit = true;
        }
      }
    }
    if (hit) conflicts.add(sgKey);
  }

  return conflicts;
}

export function collectPendingKeys(
  files: Record<string, Record<string, Record<string, string>>>,
  removals: Record<string, Record<string, string[]>>,
): Set<string> {
  const keys = new Set<string>();
  for (const sections of Object.values(files)) {
    for (const entries of Object.values(sections)) {
      for (const key of Object.keys(entries)) keys.add(key.toLowerCase());
    }
  }
  for (const sections of Object.values(removals)) {
    for (const list of Object.values(sections)) {
      for (const key of list) keys.add(key.toLowerCase());
    }
  }
  return keys;
}
