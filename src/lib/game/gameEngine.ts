import type { AppTab, GameProfile } from "@/lib/core/types";

export function supportsIniPresets(game: GameProfile): boolean {
  return !!(game.config_dir && game.is_ue);
}

export function isGameTabAvailable(game: GameProfile, tab: AppTab): boolean {
  switch (tab) {
    case "library":
      return true;
    case "advanced":
      return supportsIniPresets(game);
    default:
      return false;
  }
}

export function resolveGameTab(game: GameProfile): AppTab {
  if (supportsIniPresets(game)) return "advanced";
  return "library";
}

export function resolveGameTabRoute(game: GameProfile): "advanced" | null {
  const tab = resolveGameTab(game);
  return tab === "library" ? null : tab;
}
