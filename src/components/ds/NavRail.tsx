import type { RefObject } from "react";
import { Library, Settings, SlidersHorizontal } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useLocation, useNavigate } from "react-router-dom";
import { gameTabPath, libraryPath } from "../../lib/routes";
import { goToLibrary } from "../../lib/navigation";
import type { AppTab, GameProfile } from "../../lib/types";
import { cn } from "../../lib/cn";

interface Props {
  active: AppTab;
  selectedGame: GameProfile | null;
  settingsOpen?: boolean;
  settingsButtonRef?: RefObject<HTMLButtonElement | null>;
  onSettingsClick?: () => void;
}

export function NavRail({
  active,
  selectedGame,
  settingsOpen = false,
  settingsButtonRef,
  onSettingsClick,
}: Props) {
  const { t } = useTranslation("sidebar");
  const navigate = useNavigate();
  const location = useLocation();
  const gameTarget = selectedGame ? gameTabPath(selectedGame.id, "advanced") : libraryPath();

  const items = [
    {
      id: "library" as AppTab,
      label: t("tabs.library.label"),
      icon: Library,
      onClick: () => goToLibrary(navigate, location),
    },
    {
      id: "advanced" as AppTab,
      label: selectedGame ? selectedGame.name : t("game"),
      icon: SlidersHorizontal,
      onClick: () => {
        if (selectedGame) navigate({ pathname: gameTarget, search: "", hash: "" });
      },
      disabled: !selectedGame,
    },
  ];

  return (
    <aside className="relative z-50 flex w-16 shrink-0 flex-col items-center border-r border-[var(--color-border-strong)] bg-[var(--color-bg-soft)] py-3">
      <nav className="flex flex-1 flex-col gap-2">
        {items.map(({ id, label, icon: Icon, onClick, disabled }) => {
          const isActive = active === id || (id === "advanced" && active !== "library");
          return (
            <button
              key={id}
              type="button"
              disabled={disabled}
              aria-disabled={disabled}
              aria-current={isActive ? "page" : undefined}
              aria-label={label}
              title={label}
              onClick={(event) => {
                event.preventDefault();
                if (disabled) return;
                onClick();
              }}
              className={cn(
                "relative grid h-11 w-11 cursor-pointer place-items-center rounded-[var(--radius-panel)] border text-[var(--color-text-muted)] transition touch-manipulation",
                isActive
                  ? "border-[var(--color-accent)] bg-[var(--color-accent-soft)] text-[var(--color-accent-hover)] ring-1 ring-[var(--color-accent)]/45"
                  : "border-transparent hover:border-[var(--color-border-strong)] hover:bg-[var(--color-surface)] hover:text-[var(--color-text-secondary)]",
                disabled && "pointer-events-none opacity-35",
              )}
            >
              <Icon size={19} />
            </button>
          );
        })}
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
          "relative grid h-11 w-11 cursor-pointer place-items-center rounded-[var(--radius-panel)] border text-[var(--color-text-muted)] transition touch-manipulation",
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
