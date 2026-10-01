import type { GameParameter } from "@/lib/core/types";

function normalizeIniSection(section: string): string {
  let s = section.trim();
  if (s.startsWith("[") && s.endsWith("]")) {
    s = s.slice(1, -1);
  }
  return s.toLowerCase();
}

/** Stable id for ini row from raw file/section/key (section may be `[Name]` or `Name`). */
export function iniSnapshotKeyFromParts(file: string, section: string, key: string): string {
  return `${file.toLowerCase()}|${normalizeIniSection(section)}|${key.toLowerCase()}`;
}

/** Stable id for a parameter row in ini (file + section + key). */
export function iniSnapshotKey(
  p: Pick<GameParameter, "file" | "section" | "key">,
): string {
  return iniSnapshotKeyFromParts(p.file, p.section, p.key);
}

/** Keys that were already in ini before GSM started managing this configuration. */
export function buildIniSnapshot(parameters: GameParameter[]): Set<string> {
  const keys = new Set<string>();
  for (const p of parameters) {
    if (p.present_in_ini) keys.add(iniSnapshotKey(p));
  }
  return keys;
}

/** Keep the first baseline across editor remounts and app restarts. */
export function loadOrCreateIniSnapshot(
  gameId: string,
  configDir: string,
  parameters: GameParameter[],
): ReadonlySet<string> {
  const path = configDir.replace(/\//g, "\\").replace(/\\+$/, "").toLowerCase();
  const storageKey = `gsm-ini-baseline:v1:${JSON.stringify([gameId, path])}`;
  try {
    const stored = localStorage.getItem(storageKey);
    if (stored !== null) {
      const keys: unknown = JSON.parse(stored);
      if (Array.isArray(keys) && keys.every((key) => typeof key === "string")) {
        return new Set(keys);
      }
    }
  } catch {
    // A damaged or unavailable store must not prevent editing.
  }
  const snapshot = buildIniSnapshot(parameters);
  try {
    localStorage.setItem(storageKey, JSON.stringify([...snapshot]));
  } catch {
    // Retain the in-memory baseline when storage is unavailable.
  }
  return snapshot;
}

export function isIniShippedKey(
  p: Pick<GameParameter, "file" | "section" | "key">,
  shippedIniKeys: ReadonlySet<string>,
): boolean {
  return shippedIniKeys.has(iniSnapshotKey(p));
}

export const EMPTY_INI_SNAPSHOT: ReadonlySet<string> = new Set();
