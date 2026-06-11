import type { AppTab, GameProfile } from "./types";

/** Игры с пресетами, разобранными автором (Forza и др.) — флаг из known.json / discovery. */
export function isAuthorCuratedGame(game: GameProfile | null | undefined): boolean {
  if (!game) return false;
  return game.is_author_curated === true;
}

export const AUTHOR_CURATED_SECTION_TITLE = "Разобрано автором";

export function supportsIniPresets(game: GameProfile): boolean {
  return !!(
    game.config_dir &&
    (game.is_ue || game.is_unity || isAuthorCuratedGame(game))
  );
}

export function supportsReShade(game: GameProfile): boolean {
  return !!game.install_dir?.trim();
}

export function isGameTabAvailable(game: GameProfile, tab: AppTab): boolean {
  switch (tab) {
    case "library":
      return true;
    case "wizard":
    case "advanced":
    case "backups":
      return supportsIniPresets(game);
    case "reshade":
      return supportsReShade(game);
    default:
      return false;
  }
}

export function resolveGameTab(game: GameProfile): AppTab {
  if (supportsIniPresets(game)) return "wizard";
  if (supportsReShade(game)) return "reshade";
  return "library";
}
