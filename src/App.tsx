import { QueryClientProvider, useQuery, useQueryClient } from "@tanstack/react-query";
import { Suspense, lazy, useEffect } from "react";
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
import { useCrashReporting } from "@/hooks/app/useCrashReporting";
import { AppShell } from "@/components/layout/AppShell";
import { AppWindowFocusProvider } from "@/context/AppWindowFocusProvider";
import { AppSettingsProvider } from "@/hooks/app/useAppSettings";
import { useBackgroundSafeEnabled } from "@/hooks/app/useBackgroundSafeEnabled";
import { useSelectedGame } from "@/hooks/game/useSelectedGame";
import { scanGames } from "@/lib/api";
import { prefetchGameWorkspace, isGameTabAvailable, resolveGameTabRoute } from "@/lib/game";
import {
  gameTabPath,
  libraryPath,
  LegacyGameRouteRedirect,
  openGameEditor,
  tabFromPathname,
} from "@/lib/routing";
import { queryClient, type GameProfile } from "@/lib/core";

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

export function AppContent() {
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const location = useLocation();
  const tab = tabFromPathname(location.pathname);
  const gamesQueryEnabled = useBackgroundSafeEnabled();

  const { data: games = [], isPending: gamesLoading } = useQuery({
    queryKey: ["games"],
    queryFn: scanGames,
    enabled: gamesQueryEnabled,
    staleTime: 2 * 60_000,
  });

  const { selectedGame, gameRoute } = useSelectedGame(games);

  useEffect(() => {
    if (selectedGame && gameRoute) {
      prefetchGameWorkspace(queryClient, selectedGame);
    }
  }, [queryClient, selectedGame, gameRoute?.gameId]);

  const handleSelectGame = (game: GameProfile) => {
    if (!resolveGameTabRoute(game)) {
      return;
    }
    prefetchGameWorkspace(queryClient, game);
    openGameEditor(navigate, game.id, "basic");
  };

  const handleGameUpdated = (game: GameProfile) => {
    if (gameRoute?.gameId === game.id) {
      if (isGameTabAvailable(game, gameRoute.tab)) {
        prefetchGameWorkspace(queryClient, game);
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
            <Route
              path="/game/:gameId/backups"
              element={<LegacyGameRouteRedirect games={games} />}
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
        </Suspense>
      </ErrorBoundary>
    </AppShell>
  );
}

export default function App() {
  useCrashReporting();

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
