import GAME_RENDERING_KEY_MARKERS from "@shared/game_rendering_key_markers.json";

export function isGameRenderingKey(key: string): boolean {
  const lower = key.toLowerCase();
  return GAME_RENDERING_KEY_MARKERS.some((needle) => lower.includes(needle));
}
