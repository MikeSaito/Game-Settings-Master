import { convertFileSrc } from "@tauri-apps/api/core";
import type { GameProfile } from "./types";

const STEAM_CDN = "https://cdn.cloudflare.steamstatic.com/steam/apps";

function steamAppId(game: GameProfile): string | null {
  if (game.source !== "steam" && !game.id.startsWith("steam-")) {
    return null;
  }
  const id = game.id.startsWith("steam-") ? game.id.slice("steam-".length) : null;
  return id && /^\d+$/.test(id) ? id : null;
}

export function steamLibraryHeroUrl(appId: string): string {
  return `${STEAM_CDN}/${appId}/library_hero.jpg`;
}

/** Кандидаты для карточек: small header → large hero. */
export function resolveGameCoverCandidates(game: GameProfile): string[] {
  if (game.custom_cover) {
    return [convertFileSrc(game.custom_cover)];
  }

  const candidates: string[] = [];
  if (game.cover_url) {
    candidates.push(game.cover_url);
  }
  const appId = steamAppId(game);
  if (appId) {
    const hero = steamLibraryHeroUrl(appId);
    if (!candidates.includes(hero)) {
      candidates.push(hero);
    }
  }
  return candidates;
}

export function resolveGameCoverSrc(game: GameProfile): string | null {
  const candidates = resolveGameCoverCandidates(game);
  return candidates[0] ?? null;
}

/** Кандидаты для широкой шапки: сначала HQ hero, затем обычная обложка. */
export function resolveGameHeroCoverCandidates(game: GameProfile): string[] {
  if (game.custom_cover) {
    return [convertFileSrc(game.custom_cover)];
  }

  const candidates: string[] = [];
  const appId = steamAppId(game);
  if (appId) {
    candidates.push(steamLibraryHeroUrl(appId));
  }
  if (game.cover_url && !candidates.includes(game.cover_url)) {
    candidates.push(game.cover_url);
  }
  return candidates;
}

export function gameCoverFallbackLetter(name: string): string {
  const trimmed = name.trim();
  return trimmed ? trimmed.charAt(0).toUpperCase() : "?";
}
