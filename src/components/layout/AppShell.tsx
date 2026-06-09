import type { ReactNode } from "react";
import { GameWorkspaceProvider } from "../../context/GameWorkspaceContext";
import { GameHeroHeader } from "./GameHeroHeader";
import { Sidebar } from "./Sidebar";
import type { AppTab, GameProfile } from "../../lib/types";

interface Props {
  tab: AppTab;
  onTabChange: (tab: AppTab) => void;
  selectedGame: GameProfile | null;
  children: ReactNode;
}

export function AppShell({ tab, onTabChange, selectedGame, children }: Props) {
  const showGameChrome = tab !== "library" && !!selectedGame;

  return (
    <div className="app-bg flex h-screen overflow-hidden">
      <Sidebar
        active={tab}
        onChange={onTabChange}
        selectedGame={selectedGame}
        onGoLibrary={() => onTabChange("library")}
      />
      <main className="flex min-w-0 flex-1 flex-col overflow-hidden">
        {showGameChrome && selectedGame && (
          <GameHeroHeader
            game={selectedGame}
            activeTab={tab}
            onTabChange={onTabChange}
          />
        )}
        <div className="flex-1 overflow-y-auto">
          <div className="mx-auto max-w-6xl px-8 py-8">
            {showGameChrome && selectedGame ? (
              <GameWorkspaceProvider game={selectedGame} activeTab={tab}>
                {children}
              </GameWorkspaceProvider>
            ) : (
              children
            )}
          </div>
        </div>
      </main>
    </div>
  );
}
