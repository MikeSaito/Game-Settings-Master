import { QueryClientProvider, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";
import { UpdateGate } from "./components/UpdateGate";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { AppShell } from "./components/layout/AppShell";
import { AppWindowFocusProvider } from "./context/AppWindowFocusProvider";
import { usePresetCatalogRefresh } from "./hooks/usePresetCatalogRefresh";
import { scanGames } from "./lib/api";
import { prefetchGameWorkspace } from "./lib/prefetchGameWorkspace";
import { isGameTabAvailable, resolveGameTab } from "./lib/gameEngine";
import { queryClient } from "./lib/queryClient";
import { AdvancedEditor } from "./pages/AdvancedEditor";
import { GameLibrary } from "./pages/GameLibrary";
import { SettingsWizard } from "./pages/SettingsWizard";
import { Backups } from "./pages/Backups";
import { ReShade } from "./pages/ReShade";
import type { AppTab, GameProfile } from "./lib/types";

function AppContent() {
  usePresetCatalogRefresh();
  const queryClient = useQueryClient();
  const [tab, setTab] = useState<AppTab>("library");
  const [selectedGame, setSelectedGame] = useState<GameProfile | null>(null);
  const previousGameIdRef = useRef<string | null>(null);

  const { data: games = [] } = useQuery({
    queryKey: ["games"],
    queryFn: scanGames,
    staleTime: 2 * 60_000,
  });

  useEffect(() => {
    if (!selectedGame) return;
    const fresh = games.find((g) => g.id === selectedGame.id);
    if (!fresh) {
      setSelectedGame(null);
      setTab("library");
      return;
    }
    if (fresh !== selectedGame) {
      setSelectedGame(fresh);
    }
  }, [games, selectedGame]);

  useEffect(() => {
    if (
      selectedGame &&
      tab !== "library" &&
      isGameTabAvailable(selectedGame, tab)
    ) {
      prefetchGameWorkspace(queryClient, selectedGame, tab);
    }
  }, [queryClient, selectedGame, tab]);

  useEffect(() => {
    const currentGameId = selectedGame?.id ?? null;
    if (currentGameId === previousGameIdRef.current) return;
    previousGameIdRef.current = currentGameId;

    if (!selectedGame) {
      setTab("library");
      return;
    }

    setTab((currentTab) => {
      if (currentTab !== "library" && isGameTabAvailable(selectedGame, currentTab)) {
        return currentTab;
      }
      return resolveGameTab(selectedGame);
    });
  }, [selectedGame]);

  const handleSelectGame = (game: GameProfile) => {
    const nextTab =
      tab === "library"
        ? resolveGameTab(game)
        : isGameTabAvailable(game, tab)
          ? tab
          : resolveGameTab(game);
    prefetchGameWorkspace(queryClient, game, nextTab);
    setSelectedGame(game);
    setTab(nextTab);
  };

  const handleGameUpdated = (game: GameProfile) => {
    void queryClient.invalidateQueries({ queryKey: ["reshade-workspace", game.id] });
    void queryClient.invalidateQueries({ queryKey: ["reshade-status", game.id] });
    void queryClient.invalidateQueries({ queryKey: ["reshade-settings", game.id] });
    void queryClient.invalidateQueries({ queryKey: ["reshade-preset-details"] });

    setSelectedGame((prev) => {
      if (prev?.id !== game.id) return prev;
      const targetTab = tab === "library" ? resolveGameTab(game) : tab;
      if (targetTab !== "library" && isGameTabAvailable(game, targetTab)) {
        prefetchGameWorkspace(queryClient, game, targetTab);
      }
      return game;
    });
  };

  const handleGameRemoved = (id: string) => {
    if (selectedGame?.id === id) {
      setSelectedGame(null);
      setTab("library");
    }
  };

  return (
    <AppShell tab={tab} onTabChange={setTab} selectedGame={selectedGame}>
      <ErrorBoundary resetKey={`${tab}:${selectedGame?.id ?? ""}`}>
        {tab === "library" && (
          <GameLibrary
            selectedGame={selectedGame}
            onSelectGame={handleSelectGame}
            onGameUpdated={handleGameUpdated}
            onGameRemoved={handleGameRemoved}
          />
        )}
        {selectedGame && tab === "wizard" && (
          <SettingsWizard game={selectedGame} />
        )}
        {selectedGame && tab === "advanced" && (
          <AdvancedEditor game={selectedGame} />
        )}
        {selectedGame && tab === "backups" && (
          <Backups game={selectedGame} />
        )}
        {selectedGame && tab === "reshade" && (
          <ReShade game={selectedGame} />
        )}
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
