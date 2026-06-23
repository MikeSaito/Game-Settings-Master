import type { GameParameter } from "@/lib/core/types";

import { DLSS_MODE_TO_NUM } from "./constants";

export function findInFile(
  params: GameParameter[],
  file: string,
  key: string,
): GameParameter | undefined {
  return params.find((p) => p.file === file && p.key === key);
}

export function setInFile(
  params: GameParameter[],
  file: string,
  key: string,
  value: string,
): GameParameter[] {
  return params.map((p) =>
    p.file === file && p.key === key ? { ...p, value } : p,
  );
}

export function normalizeDlssMode(value: string): string {
  const v = value.trim();
  if (DLSS_MODE_TO_NUM[v] != null) return v;
  const lower = v.toLowerCase();
  for (const mode of Object.keys(DLSS_MODE_TO_NUM)) {
    if (mode.toLowerCase() === lower) return mode;
  }
  return v;
}

export function dlssIsOff(mode: string): boolean {
  const m = normalizeDlssMode(mode);
  return m === "Off" || m === "0";
}
