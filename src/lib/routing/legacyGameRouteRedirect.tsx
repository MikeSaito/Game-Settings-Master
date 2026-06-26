import { useEffect } from "react";
import { useLocation, useNavigate, useParams } from "react-router-dom";
import { resolveGameTabRoute } from "@/lib/game/gameEngine";
import { gameTabPath, libraryPath } from "@/lib/routing/routes";
import { writeStoredPanel } from "@/lib/routing/editorPanels";
import type { GameProfile } from "@/lib/core/types";

/** Redirect legacy `/wizard`, `/reshade`, `/backups` to `/advanced` (backups → panel state). */
export function LegacyGameRouteRedirect({ games }: { games: GameProfile[] }) {
  const { gameId = "" } = useParams();
  const location = useLocation();
  const navigate = useNavigate();
  const decodedId = decodeURIComponent(gameId);

  useEffect(() => {
    if (games.length === 0) return;
    const game = games.find((g) => g.id === decodedId);
    if (game) {
      if (location.pathname.endsWith("/backups")) {
        writeStoredPanel(game.id, "backups");
      }
      navigate(gameTabPath(game.id, resolveGameTabRoute(game) ?? "advanced"), {
        replace: true,
      });
    } else {
      navigate(libraryPath(), { replace: true });
    }
  }, [games, decodedId, location.pathname, navigate]);

  return null;
}
