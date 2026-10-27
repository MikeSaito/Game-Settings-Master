import { useRef, useState, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { useLocation } from "react-router-dom";
import { GameWorkspaceProvider } from "@/context/GameWorkspaceContext";
import { tabFromPathname } from "@/lib/routing";
import type { GameProfile } from "@/lib/core";
import { NavRail } from "@/components/ds/NavRail";
import { SettingsPanel } from "@/components/settings/SettingsPanel";
import { GameContextBar } from "./GameContextBar";

interface Props {
  selectedGame: GameProfile | null;
  children: ReactNode;
}

export function AppShell({ selectedGame, children }: Props) {
  const { t } = useTranslation("common");
  const location = useLocation();
  const settingsButtonRef = useRef<HTMLButtonElement | null>(null);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const tab = tabFromPathname(location.pathname);
  const showGameChrome = tab !== "library" && !!selectedGame;
  const isEditorWorkspace = tab === "advanced" && !!selectedGame;

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
      <div
        className={
          isEditorWorkspace
            ? "relative z-0 flex min-h-0 flex-1 flex-col overflow-hidden"
            : "relative z-0 min-h-0 flex-1 overflow-y-auto overscroll-contain"
        }
      >
        <div
          className={
            isEditorWorkspace
              ? "mx-auto flex min-h-0 w-full flex-1 flex-col px-6 py-3 max-sm:px-3"
              : "mx-auto max-w-[1400px] px-6 py-7 max-sm:px-3"
          }
        >
          {children}
        </div>
        <SettingsPanel open={settingsOpen} onClose={closeSettings} scoped />
      </div>
    </>
  );

  return (
    <div className="material-app flex h-screen gap-1 overflow-hidden py-3 pr-4 max-sm:py-1 max-sm:pr-1 bg-[var(--color-bg)] text-[var(--color-text)]">
      <a
        href="#main-content"
        className="skip-link"
        onClick={(event) => {
          event.preventDefault();
          document.getElementById("main-content")?.focus();
        }}
      >
        {t("skipToContent")}
      </a>
      <NavRail
        active={tab}
        selectedGame={selectedGame}
        settingsOpen={settingsOpen}
        settingsButtonRef={settingsButtonRef}
        onSettingsClick={() => setSettingsOpen((open) => !open)}
      />
      <main
        id="main-content"
        tabIndex={-1}
        className="relative flex min-w-0 flex-1 flex-col overflow-hidden rounded-[28px] bg-[var(--color-bg-soft)] outline-none"
      >
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
