import { useEffect, useRef } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import { isGameTabAvailable, resolveGameTabRoute } from "@/lib/game/gameEngine";
import { gameTabPath, libraryPath, parseGameRoute } from "@/lib/routing";
import type { GameProfile } from "@/lib/core/types";

/**
 * Resolves the active game from the URL and scan list.
 * Keeps the last known profile while parameters refetch (avoids editor flash).
 */
export function useSelectedGame(games: GameProfile[]) {
  const navigate = useNavigate();
  const location = useLocation();
  const gameRoute = parseGameRoute(location.pathname);
  const lastKnownGameRef = useRef<GameProfile | null>(null);

  const selectedGameFromList = gameRoute
    ? games.find((g) => g.id === gameRoute.gameId) ?? null
    : null;

  if (selectedGameFromList) {
    lastKnownGameRef.current = selectedGameFromList;
  }

  const selectedGame =
    selectedGameFromList ??
    (gameRoute && lastKnownGameRef.current?.id === gameRoute.gameId
      ? lastKnownGameRef.current
      : null);

  useEffect(() => {
    if (gameRoute && games.length > 0 && !selectedGame) {
      navigate(libraryPath(), { replace: true });
    }
  }, [gameRoute, games.length, selectedGame, navigate]);

  useEffect(() => {
    if (
      selectedGame &&
      gameRoute &&
      !isGameTabAvailable(selectedGame, gameRoute.tab)
    ) {
      navigate(gameTabPath(selectedGame.id, resolveGameTabRoute(selectedGame) ?? "advanced"), {
        replace: true,
      });
    }
  }, [selectedGame, gameRoute, navigate]);

  return { selectedGame, gameRoute };
}
