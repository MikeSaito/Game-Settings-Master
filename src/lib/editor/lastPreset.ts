interface StoredPreset {
  presetId: string;
  at: number;
}

const key = (gameId: string) => `uesm:lastPreset:${gameId}`;

export function saveLastPreset(gameId: string, presetId: string): void {
  try {
    const payload: StoredPreset = { presetId, at: Date.now() };
    localStorage.setItem(key(gameId), JSON.stringify(payload));
  } catch {
    /* ignore quota */
  }
}

export function getLastPreset(gameId: string): StoredPreset | null {
  try {
    const raw = localStorage.getItem(key(gameId));
    if (!raw) return null;
    const parsed = JSON.parse(raw) as StoredPreset;
    if (!parsed?.presetId) return null;
    return parsed;
  } catch {
    return null;
  }
}

const PRESET_LABELS: Record<string, string> = {
  potato: "Potato",
  minimum: "Minimum",
  low: "Low",
  medium: "Medium",
  high: "High",
  ultramax: "Ultra Max",
  "ultra-low": "Ultra Low",
  "ultra-high": "Ultra High",
  epic: "Epic",
};

export function formatPresetLabel(id: string): string {
  return (
    PRESET_LABELS[id] ??
    id
      .split("-")
      .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
      .join(" ")
  );
}
