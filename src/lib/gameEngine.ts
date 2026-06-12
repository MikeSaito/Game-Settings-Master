import i18n from "../i18n";
import type { AppTab, GameProfile } from "./types";

/** Games with author-curated presets (Forza, etc.) — flag from known.json / discovery. */
export function isAuthorCuratedGame(game: GameProfile | null | undefined): boolean {
  if (!game) return false;
  return game.is_author_curated === true;
}

export function authorCuratedSectionTitle(): string {
  return i18n.t("common:authorCuratedSection");
}

export function supportsIniPresets(game: GameProfile): boolean {
  return !!(
    game.config_dir &&
    (game.is_ue || game.is_unity || isAuthorCuratedGame(game))
  );
}

export function supportsReShade(game: GameProfile): boolean {
  return !!game.install_dir?.trim();
}

/** Author presets tab only for author-curated games (Forza, etc.). */
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
