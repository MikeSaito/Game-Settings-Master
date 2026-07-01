import {
  engineParamId,
  isEngineEnabled,
  isEngineToggleable,
  paramId,
} from "./engineParams";
import type { GameParameter } from "@/lib/core/types";

const ENGINE_INI_FILES = new Set(["Engine.ini", "Scalability.ini", "Game.ini"]);
const R_CVAR_RE = /r\.[A-Za-z0-9_.]+/g;

/**
 * Conflict detection uses tier_hint r.* CVars first. When a game leaves extra manual
 * r.* overrides in ini (not listed in tier_hint), prefix rules below catch the same family.
 */
export const SG_R_PREFIX: Record<string, string> = {};

/** sg.{Group}Quality → r.{group} prefix for extra override detection. */
export function sgQualityToRPrefix(sgKey: string): string | null {
  const match = sgKey.trim().toLowerCase().match(/^sg\.(.+?)quality$/);
  if (!match?.[1]) return null;
  return `r.${match[1]}`;
}

/**
 * Match r.{Group}Quality and r.{group}.* sub-cvars — not unrelated keys that merely
 * share the same substring (e.g. r.TextureStreamingPoolSize for sg.TextureQuality).
 */
export function matchesSgRPrefixFamily(key: string, prefix: string): boolean {
  const normalizedKey = key.toLowerCase();
  const normalizedPrefix = prefix.toLowerCase();
  if (!normalizedKey.startsWith(normalizedPrefix)) return false;
  const rest = normalizedKey.slice(normalizedPrefix.length);
  if (rest.length === 0) return true;
  return rest.startsWith("quality") || rest.startsWith(".");
}

export interface TierCvarPreview {
  tierLabel: string;
  cvars: Array<{ key: string; value: string }>;
}

export interface SgEngineConflictGroup {
  sgKey: string;
  sgParam: GameParameter;
  sgValue: string;
  conflictingRParams: GameParameter[];
  tierPreview: TierCvarPreview | null;
}

export function extractRelatedRCvars(tierHint: string | null | undefined): string[] {
  if (!tierHint) return [];
  const matches = tierHint.match(R_CVAR_RE) ?? [];
  return [...new Set(matches.map((m) => m.toLowerCase()))];
}

/** Parse tier A hint segment for the active sg.* quality index (e.g. "2" → High tier CVars). */
export function parseTierCvarsForSgValue(
  tierHint: string | null | undefined,
  sgValue: string,
): TierCvarPreview | null {
  if (!tierHint?.trim()) return null;
  const targetIndex = Number(sgValue.trim());
  if (!Number.isFinite(targetIndex)) return null;

  for (const segment of tierHint.split("|")) {
    const trimmed = segment.trim();
    const match = trimmed.match(/^(.+?\(\s*(\d+)\s*\))\s*:\s*(.+)$/);
    if (!match) continue;
    const index = Number(match[2]);
    if (index !== targetIndex) continue;

    const cvars = match[3]
      .split("·")
      .map((part) => part.trim())
      .filter(Boolean)
      .map((part) => {
        const eq = part.indexOf("=");
        if (eq <= 0) return null;
        return {
          key: part.slice(0, eq).trim(),
          value: part.slice(eq + 1).trim(),
        };
      })
      .filter((row): row is { key: string; value: string } => row != null);

    if (cvars.length === 0) return null;
    return { tierLabel: match[1].trim(), cvars };
  }

  return null;
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
  if (isEngineToggleable(param)) {
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

function collectRelatedRKeys(sg: GameParameter, params: GameParameter[]): Set<string> {
  const sgKey = sg.key.toLowerCase();
  const related = new Set(extractRelatedRCvars(sg.tier_hint));
  const prefixes = new Set<string>();
  const derived = sgQualityToRPrefix(sgKey);
  if (derived) prefixes.add(derived);
  const extra = SG_R_PREFIX[sgKey];
  if (extra) prefixes.add(extra.toLowerCase());

  if (prefixes.size > 0) {
    for (const param of params) {
      const key = param.key.toLowerCase();
      for (const prefix of prefixes) {
        if (matchesSgRPrefixFamily(key, prefix)) {
          related.add(key);
          break;
        }
      }
    }
  }
  return related;
}

function catalogParamId(p: Pick<GameParameter, "file" | "section" | "key">): string {
  return `${p.file.toLowerCase()}|${p.section.toLowerCase()}|${p.key.toLowerCase()}`;
}

/**
 * Detailed sg.* ↔ r.* conflict groups for actionable UI.
 */
export function analyzeSgEngineConflictGroups(
  params: GameParameter[],
  pendingKeys: Set<string>,
  engineEnabled: Set<string>,
): SgEngineConflictGroup[] {
  const byKey = paramsByKey(params);
  const groups: SgEngineConflictGroup[] = [];
  const seenSg = new Set<string>();

  for (const sg of params) {
    if (!sg.key.startsWith("sg.") || sg.file !== "GameUserSettings.ini") continue;
    if (!isSgActive(sg, pendingKeys)) continue;

    const sgKey = sg.key.toLowerCase();
    if (seenSg.has(sgKey)) continue;

    const related = collectRelatedRKeys(sg, params);
    if (related.size === 0) continue;

    const conflictingRParams: GameParameter[] = [];
    for (const rKey of related) {
      for (const rParam of byKey.get(rKey) ?? []) {
        if (!isREngineOverride(rParam, pendingKeys, engineEnabled)) continue;
        if (!conflictingRParams.some((p) => paramId(p) === paramId(rParam))) {
          conflictingRParams.push(rParam);
        }
      }
    }

    if (conflictingRParams.length === 0) continue;
    seenSg.add(sgKey);

    const sgDraft =
      params.find(
        (p) =>
          p.key.toLowerCase() === sgKey && p.file === "GameUserSettings.ini",
      ) ?? sg;

    groups.push({
      sgKey,
      sgParam: sgDraft,
      sgValue: sgDraft.value,
      conflictingRParams,
      tierPreview: parseTierCvarsForSgValue(sgDraft.tier_hint, sgDraft.value),
    });
  }

  return groups;
}

/** Disable conflicting r.* overrides and revert draft values; keep sg.* as-is. */
export function resolveConflictKeepSg(
  group: SgEngineConflictGroup,
  params: GameParameter[],
  parameters: GameParameter[],
  engineEnabled: Set<string>,
): { params: GameParameter[]; engineEnabled: Set<string> } {
  const baselineByCatalogId = new Map(
    parameters.map((p) => [catalogParamId(p), p]),
  );
  const nextEnabled = new Set(engineEnabled);
  const targetIds = new Set(group.conflictingRParams.map((p) => paramId(p)));

  for (const rParam of group.conflictingRParams) {
    if (isEngineToggleable(rParam)) {
      nextEnabled.delete(engineParamId(rParam));
    }
  }

  const nextParams = params.map((p) => {
    if (!targetIds.has(paramId(p))) return p;
    if (isEngineToggleable(p)) {
      const baseline = baselineByCatalogId.get(catalogParamId(p));
      if (!baseline) return p;
      return { ...p, value: baseline.value };
    }
    return { ...p, value: "", present_in_ini: false };
  });

  return { params: nextParams, engineEnabled: nextEnabled };
}

/**
 * Keys that overlap sg.* GUS tiers with active r.* engine overrides (ini or pending).
 */
export function detectSgEngineConflicts(
  params: GameParameter[],
  pendingKeys: Set<string>,
  engineEnabled: Set<string>,
): Set<string> {
  const conflicts = new Set<string>();

  for (const group of analyzeSgEngineConflictGroups(params, pendingKeys, engineEnabled)) {
    conflicts.add(group.sgKey);
    for (const rParam of group.conflictingRParams) {
      conflicts.add(rParam.key.toLowerCase());
    }
  }

  return conflicts;
}

export function collectPendingKeys(
  files: Record<string, Record<string, Record<string, string>>>,
  removals: Record<string, Record<string, string[]>>,
): Set<string> {
  return collectPendingWriteKeys(files, removals);
}

/** Keys with pending writes; removals are excluded (marked for delete, not active override). */
export function collectPendingWriteKeys(
  files: Record<string, Record<string, Record<string, string>>>,
  removals: Record<string, Record<string, string[]>> = {},
): Set<string> {
  const keys = new Set<string>();
  for (const sections of Object.values(files)) {
    for (const entries of Object.values(sections)) {
      for (const key of Object.keys(entries)) keys.add(key.toLowerCase());
    }
  }
  const removalKeys = new Set<string>();
  for (const sections of Object.values(removals)) {
    for (const list of Object.values(sections)) {
      for (const key of list) removalKeys.add(key.toLowerCase());
    }
  }
  for (const key of removalKeys) keys.delete(key);
  return keys;
}
