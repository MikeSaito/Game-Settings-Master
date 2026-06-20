import type { ReactNode } from "react";
import { useLocation } from "react-router-dom";
import { GameWorkspaceProvider } from "../../context/GameWorkspaceContext";
import { GameHeroHeader } from "./GameHeroHeader";
import { Sidebar } from "./Sidebar";
import { tabFromPathname } from "../../lib/routes";
import type { GameProfile } from "../../lib/types";

interface Props {
  selectedGame: GameProfile | null;
  children: ReactNode;
}

export function AppShell({ selectedGame, children }: Props) {
  const location = useLocation();
  const tab = tabFromPathname(location.pathname);
  const showGameChrome = tab !== "library" && !!selectedGame;

  return (
    <div className="app-bg flex h-screen overflow-hidden">
      <Sidebar active={tab} selectedGame={selectedGame} />
      <main className="flex min-w-0 flex-1 flex-col overflow-hidden">
        {showGameChrome ? (
          <GameWorkspaceProvider game={selectedGame} activeTab={tab}>
            <div className="flex-1 overflow-y-auto">
              <GameHeroHeader game={selectedGame} activeTab={tab} />
              <div className="mx-auto max-w-6xl animate-fade-in px-8 py-8">
                {children}
              </div>
            </div>
          </GameWorkspaceProvider>
        ) : (
          <div className="flex-1 overflow-y-auto">
            <div className="mx-auto max-w-6xl animate-fade-in px-8 py-8">
              {children}
            </div>
          </div>
        )}
      </main>
    </div>
  );
}
