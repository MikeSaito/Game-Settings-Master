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

/** Вкладка авторских пресетов есть только у игр, разобранных автором (Forza и т.п.). */
export function supportsAuthorPresets(game: GameProfile): boolean {
  return !!(game.config_dir && isAuthorCuratedGame(game));
}

export function isGameTabAvailable(game: GameProfile, tab: AppTab): boolean {
  switch (tab) {
    case "library":
      return true;
    case "wizard":
      return supportsAuthorPresets(game);
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
  if (supportsAuthorPresets(game)) return "wizard";
  if (supportsIniPresets(game)) return "advanced";
  if (supportsReShade(game)) return "reshade";
  return "library";
}
