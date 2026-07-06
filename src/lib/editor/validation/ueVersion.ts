import validationIndex from "@shared/ue-validation-index.json";

export interface UeSemver {
  major: number;
  minor: number;
  patch: number;
}

export interface ValidationKeyMeta {
  intro: string | null;
  removed: string | null;
  ue4: boolean;
  ue5: boolean;
}

const INDEX = validationIndex as Record<string, ValidationKeyMeta>;

export function parseUeSemver(raw: string | null | undefined): UeSemver | null {
  const trimmed = raw?.trim();
  if (!trimmed) return null;
  const parts = trimmed.split(".");
  const major = Number(parts[0]);
  const minor = Number(parts[1] ?? "0");
  const patch = Number(parts[2] ?? "0");
  if (!Number.isFinite(major) || !Number.isFinite(minor) || !Number.isFinite(patch)) {
    return null;
  }
  return { major, minor, patch };
}

function compareSemver(a: UeSemver, b: UeSemver): number {
  if (a.major !== b.major) return a.major - b.major;
  if (a.minor !== b.minor) return a.minor - b.minor;
  return a.patch - b.patch;
}

export function lookupValidationKey(key: string): ValidationKeyMeta | null {
  return INDEX[key.toLowerCase()] ?? null;
}

/** Mirrors `reference_applies_to_version` in src-tauri/src/catalog/version.rs */
export function keyAppliesToGame(
  key: string,
  engineFamily: string | undefined,
  engineVersion: string | null | undefined,
): boolean {
  const meta = lookupValidationKey(key);
  if (!meta) return true;

  const gameVersion = parseUeSemver(engineVersion);
  if (gameVersion) {
    if (meta.intro) {
      const intro = parseUeSemver(meta.intro);
      if (intro && compareSemver(gameVersion, intro) < 0) return false;
    }
    if (meta.removed) {
      const removed = parseUeSemver(meta.removed);
      if (removed && compareSemver(gameVersion, removed) >= 0) return false;
    }
    return true;
  }

  const isUe4 = engineFamily?.toLowerCase() === "ue4";
  return isUe4 ? meta.ue4 : meta.ue5;
}
