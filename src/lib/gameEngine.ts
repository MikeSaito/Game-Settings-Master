import type { GameProfile } from "./types";

/** Игры с пресетами, разобранными автором (Forza и др.) — флаг из known.json / discovery. */
export function isAuthorCuratedGame(game: GameProfile | null | undefined): boolean {
  if (!game) return false;
  return game.is_author_curated === true;
}

export const AUTHOR_CURATED_SECTION_TITLE = "Разобрано автором";
