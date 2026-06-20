import type { AppTab, GameProfile, GameTabRoute } from "./types";

export function supportsIniPresets(game: GameProfile): boolean {
  return !!(game.config_dir && (game.is_ue || game.is_unity));
}

export function isGameTabAvailable(game: GameProfile, tab: AppTab): boolean {
  switch (tab) {
    case "library":
      return true;
    case "advanced":
    case "backups":
      return supportsIniPresets(game);
    default:
      return false;
  }
}

export function resolveGameTab(game: GameProfile): AppTab {
  if (supportsIniPresets(game)) return "advanced";
  return "library";
}

export function resolveGameTabRoute(game: GameProfile): GameTabRoute | null {
  const tab = resolveGameTab(game);
  return tab === "library" ? null : tab;
}
