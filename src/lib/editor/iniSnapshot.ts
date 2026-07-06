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

/** Keys that were already in ini on the first successful load of this game session. */
export function buildIniSnapshot(parameters: GameParameter[]): Set<string> {
  const keys = new Set<string>();
  for (const p of parameters) {
    if (p.present_in_ini) keys.add(iniSnapshotKey(p));
  }
  return keys;
}

export function isIniShippedKey(
  p: Pick<GameParameter, "file" | "section" | "key">,
  shippedIniKeys: ReadonlySet<string>,
): boolean {
  return shippedIniKeys.has(iniSnapshotKey(p));
}

export const EMPTY_INI_SNAPSHOT: ReadonlySet<string> = new Set();
