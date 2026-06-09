import { QueryClientProvider, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { UpdateGate } from "./components/UpdateGate";
import { AppShell } from "./components/layout/AppShell";
import { AppWindowFocusProvider } from "./context/AppWindowFocusProvider";
import { usePresetCatalogRefresh } from "./hooks/usePresetCatalogRefresh";
import { scanGames } from "./lib/api";
import { prefetchGameWorkspace } from "./lib/prefetchGameWorkspace";
import { isAuthorCuratedGame } from "./lib/gameEngine";
import { queryClient } from "./lib/queryClient";
import { AdvancedEditor } from "./pages/AdvancedEditor";
import { GameLibrary } from "./pages/GameLibrary";
import { SettingsWizard } from "./pages/SettingsWizard";
import { Backups } from "./pages/Backups";
import type { AppTab, GameProfile } from "./lib/types";

function resolveGameTab(game: GameProfile): AppTab {
  if (
    game.config_dir &&
    (game.is_ue || game.is_unity || isAuthorCuratedGame(game))
  ) {
    return "wizard";
  }
  return "library";
}

function AppContent() {
  usePresetCatalogRefresh();
  const queryClient = useQueryClient();
  const [tab, setTab] = useState<AppTab>("library");
  const [selectedGame, setSelectedGame] = useState<GameProfile | null>(null);

  useEffect(() => {
    void queryClient.prefetchQuery({
      queryKey: ["games"],
      queryFn: scanGames,
      staleTime: 2 * 60_000,
    });
  }, [queryClient]);

  useEffect(() => {
    if (selectedGame && tab !== "library") {
      prefetchGameWorkspace(queryClient, selectedGame, tab);
    }
  }, [queryClient, selectedGame, tab]);

  const handleSelectGame = (game: GameProfile) => {
    const nextTab = tab !== "library" ? tab : resolveGameTab(game);
    prefetchGameWorkspace(queryClient, game, nextTab);
    setSelectedGame(game);
    setTab((prev) => (prev !== "library" ? prev : resolveGameTab(game)));
  };

  const handleGameUpdated = (game: GameProfile) => {
    setSelectedGame((prev) => (prev?.id === game.id ? game : prev));
    if (game.config_dir && tab !== "library") {
      prefetchGameWorkspace(queryClient, game, tab);
    }
  };

  return (
    <AppShell tab={tab} onTabChange={setTab} selectedGame={selectedGame}>
      {tab === "library" && (
        <GameLibrary
          selectedGame={selectedGame}
          onSelectGame={handleSelectGame}
          onGameUpdated={handleGameUpdated}
        />
      )}
      {tab === "wizard" && <SettingsWizard game={selectedGame} />}
      {tab === "advanced" && <AdvancedEditor game={selectedGame} />}
      {tab === "backups" && <Backups game={selectedGame} />}
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
