import type { AppTab, GameTabRoute } from "../core/types";

export type { GameTabRoute };

export function libraryPath(): string {
  return "/library";
}

export function gameTabPath(gameId: string, tab: GameTabRoute): string {
  const id = encodeURIComponent(gameId);
  if (tab === "backups") {
    return `/game/${id}/advanced`;
  }
  return `/game/${id}/${tab}`;
}

export function parseGameRoute(pathname: string): {
  gameId: string;
  tab: GameTabRoute;
} | null {
  const match = pathname.match(/^\/game\/([^/]+)\/(advanced|backups)$/);
  if (!match) return null;
  return { gameId: decodeURIComponent(match[1]), tab: match[2] as GameTabRoute };
}

/** Removed tabs (wizard, reshade) — redirect to advanced when game exists. */
export function parseLegacyGameRoute(pathname: string): { gameId: string } | null {
  const match = pathname.match(/^\/game\/([^/]+)\/(wizard|reshade)$/);
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
