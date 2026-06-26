import type { MouseEvent, RefObject } from "react";
import { Library, Settings, SlidersHorizontal } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Link, useLocation } from "react-router-dom";
import { gameTabPath, libraryPath } from "@/lib/routing";
import type { AppTab, GameProfile } from "@/lib/core";
import { cn } from "@/lib/core";

interface Props {
  active: AppTab;
  selectedGame: GameProfile | null;
  settingsOpen?: boolean;
  settingsButtonRef?: RefObject<HTMLButtonElement | null>;
  onSettingsClick?: () => void;
}

const navButtonClass =
  "relative grid h-11 w-11 cursor-pointer place-items-center rounded-[var(--radius-panel)] border text-[var(--color-text-muted)] transition touch-manipulation";

export function NavRail({
  active,
  selectedGame,
  settingsOpen = false,
  settingsButtonRef,
  onSettingsClick,
}: Props) {
  const { t } = useTranslation("sidebar");
  const location = useLocation();
  const libraryTarget = libraryPath();
  const gameTarget = selectedGame ? gameTabPath(selectedGame.id, "advanced") : libraryTarget;

  const scrollLibraryIfActive = (event: MouseEvent) => {
    if (
      location.pathname === libraryTarget &&
      !location.hash &&
      !location.search
    ) {
      event.preventDefault();
      window.scrollTo({ top: 0, behavior: "smooth" });
    }
  };

  return (
    <aside className="relative z-50 flex w-16 shrink-0 flex-col items-center border-r border-[var(--color-border-strong)] bg-[var(--color-bg-soft)] py-3">
      <nav className="flex flex-1 flex-col gap-2">
        <Link
          to={libraryTarget}
          aria-current={active === "library" ? "page" : undefined}
          aria-label={t("tabs.library.label")}
          title={t("tabs.library.label")}
          onClick={scrollLibraryIfActive}
          className={cn(
            navButtonClass,
            active === "library"
              ? "border-[var(--color-accent)] bg-[var(--color-accent-soft)] text-[var(--color-accent-hover)] ring-1 ring-[var(--color-accent)]/45"
              : "border-transparent hover:border-[var(--color-border-strong)] hover:bg-[var(--color-surface)] hover:text-[var(--color-text-secondary)]",
          )}
        >
          <Library size={19} />
        </Link>

        {selectedGame ? (
          <Link
            to={gameTarget}
            aria-current={active !== "library" ? "page" : undefined}
            aria-label={selectedGame.name}
            title={selectedGame.name}
            className={cn(
              navButtonClass,
              active !== "library"
                ? "border-[var(--color-accent)] bg-[var(--color-accent-soft)] text-[var(--color-accent-hover)] ring-1 ring-[var(--color-accent)]/45"
                : "border-transparent hover:border-[var(--color-border-strong)] hover:bg-[var(--color-surface)] hover:text-[var(--color-text-secondary)]",
            )}
          >
            <SlidersHorizontal size={19} />
          </Link>
        ) : (
          <button
            type="button"
            disabled
            aria-disabled
            aria-label={t("game")}
            title={t("game")}
            className={cn(navButtonClass, "pointer-events-none opacity-35")}
          >
            <SlidersHorizontal size={19} />
          </button>
        )}
      </nav>
      <button
        ref={settingsButtonRef}
        type="button"
        onClick={() => onSettingsClick?.()}
        title={t("tabs.settings.label")}
        aria-label={t("tabs.settings.label")}
        aria-pressed={settingsOpen}
        aria-current={settingsOpen ? "page" : undefined}
        className={cn(
          navButtonClass,
          settingsOpen
            ? "border-[var(--color-accent)] bg-[var(--color-accent-soft)] text-[var(--color-accent-hover)] ring-1 ring-[var(--color-accent)]/45"
            : "border-transparent hover:border-[var(--color-border-strong)] hover:bg-[var(--color-surface)] hover:text-[var(--color-text-secondary)]",
        )}
      >
        <Settings size={19} />
      </button>
    </aside>
  );
}
