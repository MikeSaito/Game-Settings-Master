import type { GameParameter } from "../core/types";

export type EditorPanel = "basic" | "advanced" | "backups";

/** Parameter list filter mode in the editor sidebar. */
export type EditorFilterMode = "recommended" | "full" | "ini_only";

const GUS = "GameUserSettings.ini";

function matchesSearch(p: GameParameter, q: string): boolean {
  return (
    p.key.toLowerCase().includes(q) ||
    p.title.toLowerCase().includes(q) ||
    p.description.toLowerCase().includes(q) ||
    (p.value_hint?.toLowerCase().includes(q) ?? false)
  );
}

function normalizePanel(raw: string | null): EditorPanel | null {
  if (raw === "basic" || raw === "scalability") return "basic";
  if (raw === "advanced" || raw === "engine") return "advanced";
  if (raw === "backups") return "backups";
  return null;
}

function normalizeFilterMode(raw: string | null, panel: EditorPanel): EditorFilterMode | null {
  if (raw === "recommended" || raw === "full" || raw === "ini_only") return raw;
  if (raw === "1") return "recommended";
  if (raw === "0") return panel === "advanced" ? "full" : "ini_only";
  return null;
}

/** Basic is the in-game menu layer: GameUserSettings.ini and sg.* only. */
export function panelForParameter(p: GameParameter): EditorPanel {
  if (p.file === GUS) return "basic";
  return "advanced";
}

export function filterParamsByPanel(
  params: GameParameter[],
  panel: EditorPanel,
): GameParameter[] {
  if (panel === "backups") return [];
  return params.filter((p) => panelForParameter(p) === panel);
}

export function isRecommendedParam(
  p: GameParameter,
  panel: EditorPanel,
): boolean {
  if (panel === "basic") {
    if (p.key.startsWith("sg.")) return true;
    if (p.file === GUS && p.known) return true;
    return p.file === GUS && p.catalog_recommended === true;
  }
  return p.catalog_recommended === true;
}

/**
 * Recommended / full version catalog / ini-only (+ search bypass).
 */
export function filterParamsByMode(
  params: GameParameter[],
  mode: EditorFilterMode,
  panel: EditorPanel,
  search: string,
): GameParameter[] {
  const q = search.trim().toLowerCase();
  const searchMatch = (p: GameParameter) => q.length > 0 && matchesSearch(p, q);

  switch (mode) {
    case "ini_only":
      return params.filter((p) => p.present_in_ini || searchMatch(p));
    case "recommended":
      return params.filter((p) => isRecommendedParam(p, panel) || searchMatch(p));
    case "full":
      return params.filter((p) => searchMatch(p) || true);
    default:
      return params;
  }
}

export function defaultFilterMode(panel: EditorPanel): EditorFilterMode {
  if (panel === "backups") return "recommended";
  return panel === "basic" ? "recommended" : "full";
}

export function defaultCategoryForPanel(panel: EditorPanel): string {
  if (panel === "backups") return "All";
  return panel === "basic" ? "Scalability" : "Rendering";
}

export function panelStorageKey(gameId: string): string {
  return `gsm-editor-panel:${gameId}`;
}

export function legacyPanelStorageKey(gameId: string): string {
  return `gsm-advanced-panel:${gameId}`;
}

export function filterStorageKey(gameId: string, panel: EditorPanel): string {
  return `gsm-editor-filter:${panel}:${gameId}`;
}

export function legacyRecommendedStorageKey(gameId: string): string {
  return `gsm-advanced-recommended:${gameId}`;
}

export function engineWarningAckKey(gameId: string): string {
  return `gsm-engine-warning-ack:${gameId}`;
}

export function readStoredPanel(gameId: string): EditorPanel | null {
  try {
    const stored = normalizePanel(sessionStorage.getItem(panelStorageKey(gameId)));
    if (stored) return stored;
    const legacy = normalizePanel(sessionStorage.getItem(legacyPanelStorageKey(gameId)));
    if (legacy) {
      writeStoredPanel(gameId, legacy);
      return legacy;
    }
  } catch {
    /* ignore */
  }
  return null;
}

export function writeStoredPanel(gameId: string, panel: EditorPanel): void {
  try {
    sessionStorage.setItem(panelStorageKey(gameId), panel);
  } catch {
    /* ignore */
  }
}

export function readStoredFilterMode(
  gameId: string,
  panel: EditorPanel,
): EditorFilterMode | null {
  try {
    const raw = sessionStorage.getItem(filterStorageKey(gameId, panel));
    const parsed = normalizeFilterMode(raw, panel);
    if (parsed) return parsed;
    const legacy = sessionStorage.getItem(legacyRecommendedStorageKey(gameId));
    return normalizeFilterMode(legacy, panel);
  } catch {
    /* ignore */
  }
  return null;
}

export function writeStoredFilterMode(
  gameId: string,
  panel: EditorPanel,
  mode: EditorFilterMode,
): void {
  try {
    sessionStorage.setItem(filterStorageKey(gameId, panel), mode);
  } catch {
    /* ignore */
  }
}

export function panelFromHash(hash = ""): EditorPanel | null {
  const raw =
    hash ||
    (typeof window !== "undefined" ? window.location.hash : "");
  return normalizePanel(raw.replace(/^#/, "").toLowerCase());
}

