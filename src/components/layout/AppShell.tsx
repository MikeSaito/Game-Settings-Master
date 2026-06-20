import { useRef, useState, type ReactNode } from "react";
import { useLocation } from "react-router-dom";
import { GameWorkspaceProvider } from "../../context/GameWorkspaceContext";
import { tabFromPathname } from "../../lib/routes";
import type { GameProfile } from "../../lib/types";
import { NavRail } from "../ds/NavRail";
import { SettingsPanel } from "../settings/SettingsPanel";
import { GameContextBar } from "./GameContextBar";

interface Props {
  selectedGame: GameProfile | null;
  children: ReactNode;
}

export function AppShell({ selectedGame, children }: Props) {
  const location = useLocation();
  const settingsButtonRef = useRef<HTMLButtonElement | null>(null);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const tab = tabFromPathname(location.pathname);
  const showGameChrome = tab !== "library" && !!selectedGame;

  const closeSettings = () => {
    setSettingsOpen(false);
    settingsButtonRef.current?.focus();
  };

  const mainContent = (
    <>
      {showGameChrome && selectedGame && (
        <header className="relative z-40 shrink-0 bg-[var(--color-bg-soft)]">
          <GameContextBar game={selectedGame} />
        </header>
      )}
      <div className="relative z-0 min-h-0 flex-1 overflow-y-auto overscroll-contain">
        <div className="mx-auto max-w-[1400px] px-4 py-4">{children}</div>
        <SettingsPanel open={settingsOpen} onClose={closeSettings} scoped />
      </div>
    </>
  );

  return (
    <div className="flex h-screen overflow-hidden bg-[var(--color-bg)] text-[var(--color-text)]">
      <NavRail
        active={tab}
        selectedGame={selectedGame}
        settingsOpen={settingsOpen}
        settingsButtonRef={settingsButtonRef}
        onSettingsClick={() => setSettingsOpen((open) => !open)}
      />
      <main className="relative flex min-w-0 flex-1 flex-col overflow-hidden">
        {showGameChrome && selectedGame ? (
          <GameWorkspaceProvider game={selectedGame} activeTab={tab}>
            {mainContent}
          </GameWorkspaceProvider>
        ) : (
          mainContent
        )}
      </main>
    </div>
  );
}
