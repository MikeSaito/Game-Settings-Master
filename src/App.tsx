import { QueryClientProvider, useQuery, useQueryClient } from "@tanstack/react-query";
import { Suspense, lazy, useEffect, useRef } from "react";
import {
  Navigate,
  Route,
  Routes,
  useLocation,
  useNavigate,
  useParams,
} from "react-router-dom";
import { UpdateGate } from "./components/UpdateGate";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { AppShell } from "./components/layout/AppShell";
import { AppWindowFocusProvider } from "./context/AppWindowFocusProvider";
import { AppSettingsProvider } from "./hooks/useAppSettings";
import { useBackgroundSafeEnabled } from "./hooks/useBackgroundSafeEnabled";
import { scanGames } from "./lib/api";
import { prefetchGameWorkspace } from "./lib/prefetchGameWorkspace";
import { isGameTabAvailable, resolveGameTabRoute } from "./lib/gameEngine";
import {
  gameTabPath,
  libraryPath,
  parseGameRoute,
  tabFromPathname,
} from "./lib/routes";
import { queryClient } from "./lib/queryClient";
import { LegacyGameRouteRedirect } from "./lib/legacyGameRouteRedirect";
import type { GameProfile } from "./lib/types";

const AdvancedEditor = lazy(() =>
  import("./pages/AdvancedEditor").then((module) => ({
    default: module.AdvancedEditor,
  })),
);
const GameLibrary = lazy(() =>
  import("./pages/GameLibrary").then((module) => ({
    default: module.GameLibrary,
  })),
);

function GameEditorPage({ games }: { games: GameProfile[] }) {
  const { gameId = "" } = useParams();
  const game = games.find((g) => g.id === decodeURIComponent(gameId)) ?? null;
  if (!game) return null;
  return <AdvancedEditor game={game} />;
}

function BackupsRouteRedirect() {
  const { gameId = "" } = useParams();
  return (
    <Navigate
      to={`${gameTabPath(decodeURIComponent(gameId), "advanced")}#backups`}
      replace
    />
  );
}

export function AppContent() {
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const location = useLocation();
  const tab = tabFromPathname(location.pathname);
  const gameRoute = parseGameRoute(location.pathname);
  const previousGameIdRef = useRef<string | null>(null);
  const lastKnownGameRef = useRef<GameProfile | null>(null);
  const gamesQueryEnabled = useBackgroundSafeEnabled();

  const { data: games = [] } = useQuery({
    queryKey: ["games"],
    queryFn: scanGames,
    enabled: gamesQueryEnabled,
    staleTime: 2 * 60_000,
  });

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
      gameRoute.tab === "backups"
    ) {
      navigate(`${gameTabPath(selectedGame.id, "advanced")}#backups`, { replace: true });
      return;
    }
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
        <Suspense fallback={null}>
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
            <Route path="/game/:gameId/advanced" element={<GameEditorPage games={games} />} />
            <Route path="/game/:gameId/backups" element={<BackupsRouteRedirect />} />
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
        </Suspense>
      </ErrorBoundary>
    </AppShell>
  );
}

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <UpdateGate>
        <AppSettingsProvider>
          <AppWindowFocusProvider>
            <AppContent />
          </AppWindowFocusProvider>
        </AppSettingsProvider>
      </UpdateGate>
    </QueryClientProvider>
  );
}
