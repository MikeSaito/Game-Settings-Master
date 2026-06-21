import { useEffect } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { resolveGameTabRoute } from "./gameEngine";
import { gameTabPath, libraryPath } from "./routes";
import type { GameProfile } from "./types";

/** Redirect old /wizard and /reshade URLs to advanced (or library if game unknown). */
export function LegacyGameRouteRedirect({ games }: { games: GameProfile[] }) {
  const { gameId = "" } = useParams();
  const navigate = useNavigate();
  const decodedId = decodeURIComponent(gameId);

  useEffect(() => {
    if (games.length === 0) return;
    const game = games.find((g) => g.id === decodedId);
    if (game) {
      navigate(
        gameTabPath(game.id, resolveGameTabRoute(game) ?? "advanced"),
        { replace: true },
      );
    } else {
      navigate(libraryPath(), { replace: true });
    }
  }, [games, decodedId, navigate]);

  return null;
}
