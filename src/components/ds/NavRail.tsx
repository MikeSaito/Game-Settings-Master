import { resolveTheme } from "@/lib/settings";
import { useAppSettings } from "@/hooks/app/useAppSettings";
import { useEffect, useState } from "react";
import type { MouseEvent, RefObject } from "react";
import { Library, Settings, SlidersHorizontal, Moon, Sun } from "lucide-react";
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
  "material-rail-button relative grid h-12 w-16 cursor-pointer place-items-center rounded-full text-[var(--color-text-muted)] transition-colors touch-manipulation";

export function NavRail({
  active,
  selectedGame,
  settingsOpen = false,
  settingsButtonRef,
  onSettingsClick,
}: Props) {
  const { t } = useTranslation("sidebar");
  const location = useLocation();
  const { settings, setTheme } = useAppSettings();
  const { t: ts } = useTranslation("settings");
  const [resolvedTheme, setResolvedTheme] = useState(() =>
    resolveTheme(settings.theme),
  );
  useEffect(() => {
    const sync = () => setResolvedTheme(resolveTheme(settings.theme));
    sync();
    const media = window.matchMedia?.("(prefers-color-scheme: light)");
    media?.addEventListener?.("change", sync);
    return () => media?.removeEventListener?.("change", sync);
  }, [settings.theme]);
  const libraryTarget = libraryPath();
  const gameTarget = selectedGame
    ? gameTabPath(selectedGame.id, "advanced")
    : libraryTarget;

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
    <aside className="relative z-50 flex w-[88px] shrink-0 flex-col items-center py-2 max-sm:w-14">
      <Link
        to={libraryTarget}
        aria-label="Game Settings Master"
        className="mb-7 grid h-14 w-14 place-items-center rounded-2xl max-sm:w-12"
      >
        <img src="/logo.png" alt="" className="h-10 w-10 rounded-xl" />
      </Link>
      <nav className="flex flex-1 flex-col gap-3">
        <Link
          to={libraryTarget}
          aria-current={active === "library" ? "page" : undefined}
          aria-label={t("tabs.library.label")}
          title={t("tabs.library.label")}
          onClick={scrollLibraryIfActive}
          className={cn(
            navButtonClass,
            active === "library"
              ? "bg-[var(--color-secondary-container)] text-[var(--color-on-secondary-container)]"
              : "hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)]",
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
                ? "bg-[var(--color-secondary-container)] text-[var(--color-on-secondary-container)]"
                : "hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)]",
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
        type="button"
        className={cn(navButtonClass, "mb-3")}
        aria-label={ts(
          resolvedTheme === "light" ? "theme.dark" : "theme.light",
        )}
        title={ts(resolvedTheme === "light" ? "theme.dark" : "theme.light")}
        onClick={() =>
          setTheme(
            document.documentElement.dataset.theme === "light"
              ? "dark"
              : "light",
          )
        }
      >
        {resolvedTheme === "light" ? <Moon size={20} /> : <Sun size={20} />}
      </button>
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
            ? "bg-[var(--color-secondary-container)] text-[var(--color-on-secondary-container)]"
            : "hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)]",
        )}
      >
        <Settings size={19} />
      </button>
    </aside>
  );
}
