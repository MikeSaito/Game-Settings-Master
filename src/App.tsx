import { QueryClientProvider, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef } from "react";
import {
  Navigate,
  Route,
  Routes,
  useLocation,
  useNavigate,
} from "react-router-dom";
import { UpdateGate } from "./components/UpdateGate";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { AppShell } from "./components/layout/AppShell";
import { AppWindowFocusProvider } from "./context/AppWindowFocusProvider";
import { scanGames, setBackendLanguage } from "./lib/api";
import { currentLanguage } from "./i18n";
import { prefetchGameWorkspace } from "./lib/prefetchGameWorkspace";
import { isGameTabAvailable, resolveGameTabRoute } from "./lib/gameEngine";
import {
  gameTabPath,
  libraryPath,
  parseGameRoute,
  tabFromPathname,
} from "./lib/routes";
import { queryClient } from "./lib/queryClient";
import { AdvancedEditor } from "./pages/AdvancedEditor";
import { GameLibrary } from "./pages/GameLibrary";
import { Backups } from "./pages/Backups";
import { LegacyGameRouteRedirect } from "./lib/legacyGameRouteRedirect";
import type { GameProfile } from "./lib/types";

export function AppContent() {
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const location = useLocation();
  const tab = tabFromPathname(location.pathname);
  const gameRoute = parseGameRoute(location.pathname);
  const previousGameIdRef = useRef<string | null>(null);

  useEffect(() => {
    void setBackendLanguage(currentLanguage()).catch(() => {});
  }, []);

  const { data: games = [] } = useQuery({
    queryKey: ["games"],
    queryFn: scanGames,
    staleTime: 2 * 60_000,
  });

  const selectedGame = gameRoute
    ? games.find((g) => g.id === gameRoute.gameId) ?? null
    : null;

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

  useEffect(() => {
    if (selectedGame && gameRoute) {
      prefetchGameWorkspace(queryClient, selectedGame, gameRoute.tab);
    }
  }, [queryClient, selectedGame, gameRoute?.tab]);

  useEffect(() => {
    const currentGameId = selectedGame?.id ?? null;
    if (currentGameId === previousGameIdRef.current) return;
    previousGameIdRef.current = currentGameId;
  }, [selectedGame?.id]);

  const handleSelectGame = (game: GameProfile) => {
    const nextTab = resolveGameTabRoute(game);
    if (!nextTab) {
      navigate(libraryPath());
      return;
    }
    prefetchGameWorkspace(queryClient, game, nextTab);
    navigate(gameTabPath(game.id, nextTab));
  };

  const handleGameUpdated = (game: GameProfile) => {
    if (gameRoute?.gameId === game.id) {
      if (isGameTabAvailable(game, gameRoute.tab)) {
        prefetchGameWorkspace(queryClient, game, gameRoute.tab);
      } else {
        navigate(gameTabPath(game.id, resolveGameTabRoute(game) ?? "advanced"), {
          replace: true,
        });
      }
    }
  };

  const handleGameRemoved = (id: string) => {
    if (gameRoute?.gameId === id) {
      navigate(libraryPath(), { replace: true });
    }
  };

  return (
    <AppShell selectedGame={selectedGame}>
      <ErrorBoundary resetKey={`${tab}:${selectedGame?.id ?? ""}`}>
        <Routes>
          <Route path="/" element={<Navigate to={libraryPath()} replace />} />
          <Route
            path="/library"
            element={
              <GameLibrary
                selectedGame={selectedGame}
                onSelectGame={handleSelectGame}
                onGameUpdated={handleGameUpdated}
                onGameRemoved={handleGameRemoved}
              />
            }
          />
          <Route
            path="/game/:gameId/advanced"
            element={
              selectedGame ? <AdvancedEditor game={selectedGame} /> : null
            }
          />
          <Route
            path="/game/:gameId/backups"
            element={selectedGame ? <Backups game={selectedGame} /> : null}
          />
          <Route
            path="/game/:gameId/wizard"
            element={<LegacyGameRouteRedirect games={games} />}
          />
          <Route
            path="/game/:gameId/reshade"
            element={<LegacyGameRouteRedirect games={games} />}
          />
          <Route path="*" element={<Navigate to={libraryPath()} replace />} />
        </Routes>
      </ErrorBoundary>
    </AppShell>
  );
}

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <UpdateGate>
        <AppWindowFocusProvider>
          <AppContent />
        </AppWindowFocusProvider>
      </UpdateGate>
    </QueryClientProvider>
  );
}
