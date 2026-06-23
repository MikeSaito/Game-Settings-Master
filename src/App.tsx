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
import { UpdateGate } from "@/components/app/UpdateGate";
import { ErrorBoundary } from "@/components/app/ErrorBoundary";
import { RouteLoading } from "@/components/app/RouteLoading";
import { AppShell } from "@/components/layout/AppShell";
import { AppWindowFocusProvider } from "./context/AppWindowFocusProvider";
import { AppSettingsProvider } from "@/hooks/app/useAppSettings";
import { useBackgroundSafeEnabled } from "@/hooks/app/useBackgroundSafeEnabled";
import { scanGames } from "@/lib/api";
import { prefetchGameWorkspace } from "@/lib/game";
import { isGameTabAvailable, resolveGameTabRoute } from "@/lib/game";
import {
  gameTabPath,
  libraryPath,
  parseGameRoute,
  tabFromPathname,
} from "@/lib/routing";
import { queryClient } from "@/lib/core";
import { writeStoredPanel } from "@/lib/routing";
import { LegacyGameRouteRedirect } from "@/lib/routing";
import type { GameProfile } from "@/lib/core";

const AdvancedEditor = lazy(() =>
  import("@/pages/AdvancedEditor").then((module) => ({
    default: module.AdvancedEditor,
  })),
);
const GameLibrary = lazy(() =>
  import("@/pages/GameLibrary").then((module) => ({
    default: module.GameLibrary,
  })),
);

function GameEditorPage({
  games,
  gamesLoading,
}: {
  games: GameProfile[];
  gamesLoading: boolean;
}) {
  const { gameId = "" } = useParams();
  const id = decodeURIComponent(gameId);
  const game = games.find((g) => g.id === id) ?? null;
  if (!game) {
    if (gamesLoading) return <RouteLoading />;
    return null;
  }
  return <AdvancedEditor game={game} />;
}

function BackupsRouteRedirect() {
  const { gameId = "" } = useParams();
  const id = decodeURIComponent(gameId);
  useEffect(() => {
    writeStoredPanel(id, "backups");
  }, [id]);
  return <Navigate to={gameTabPath(id, "advanced")} replace />;
}

export function AppContent() {
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const location = useLocation();
  const tab = tabFromPathname(location.pathname);
  const gameRoute = parseGameRoute(location.pathname);
  const lastKnownGameRef = useRef<GameProfile | null>(null);
  const gamesQueryEnabled = useBackgroundSafeEnabled();

  const { data: games = [], isPending: gamesLoading } = useQuery({
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
        <Suspense fallback={<RouteLoading />}>
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
              element={<GameEditorPage games={games} gamesLoading={gamesLoading} />}
            />
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
