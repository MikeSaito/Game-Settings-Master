import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState } from "react";
import { AppShell } from "./components/layout/AppShell";
import { usePresetCatalogRefresh } from "./hooks/usePresetCatalogRefresh";
import { AdvancedEditor } from "./pages/AdvancedEditor";
import { GameLibrary } from "./pages/GameLibrary";
import { SettingsWizard } from "./pages/SettingsWizard";
import { Backups } from "./pages/Backups";
import { isAuthorCuratedGame } from "./lib/gameEngine";
import type { AppTab, GameProfile } from "./lib/types";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 30_000,
      retry: 1,
    },
  },
});

function AppContent() {
  usePresetCatalogRefresh();
  const [tab, setTab] = useState<AppTab>("library");
  const [selectedGame, setSelectedGame] = useState<GameProfile | null>(null);

  const handleSelectGame = (game: GameProfile) => {
    setSelectedGame(game);
    setTab((prev) => {
      if (prev !== "library") return prev;
      if (
        game.config_dir &&
        (game.is_ue || game.is_unity || isAuthorCuratedGame(game))
      ) {
        return "wizard";
      }
      return "library";
    });
  };

  const handleGameUpdated = (game: GameProfile) => {
    setSelectedGame((prev) => (prev?.id === game.id ? game : prev));
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
      <AppContent />
    </QueryClientProvider>
  );
}
