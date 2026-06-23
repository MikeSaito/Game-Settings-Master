import type { AppTab, GameTabRoute } from "@/lib/core/types";

export type { GameTabRoute };

export function libraryPath(): string {
  return "/library";
}

export function gameTabPath(gameId: string, _tab: GameTabRoute = "advanced"): string {
  return `/game/${encodeURIComponent(gameId)}/advanced`;
}

export function parseGameRoute(pathname: string): {
  gameId: string;
  tab: GameTabRoute;
} | null {
  const match = pathname.match(/^\/game\/([^/]+)\/advanced$/);
  if (!match) return null;
  return { gameId: decodeURIComponent(match[1]), tab: "advanced" };
}

/** Legacy game URLs (wizard, reshade, backups) — redirect to `/advanced`. */
export function parseLegacyGameRoute(pathname: string): { gameId: string } | null {
  const match = pathname.match(/^\/game\/([^/]+)\/(wizard|reshade|backups)$/);
  if (!match) return null;
  return { gameId: decodeURIComponent(match[1]) };
}

export function isLibraryRoute(pathname: string): boolean {
  return pathname === "/" || pathname === "/library";
}

export function tabFromPathname(pathname: string): AppTab {
  if (isLibraryRoute(pathname)) return "library";
  const parsed = parseGameRoute(pathname);
  return parsed?.tab ?? "library";
}
