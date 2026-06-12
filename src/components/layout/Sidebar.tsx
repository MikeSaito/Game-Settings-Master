import { Monitor } from "lucide-react";
import { useTranslation } from "react-i18next";
import { supportsIniPresets, supportsReShade } from "../../lib/gameEngine";
import type { AppTab, GameProfile } from "../../lib/types";
import { cn } from "../../lib/cn";
import { GameCover } from "../GameCover";
import { Badge } from "../ui/Badge";
import { LanguageToggle } from "./LanguageToggle";

interface Props {
  active: AppTab;
  onChange: (tab: AppTab) => void;
  selectedGame: GameProfile | null;
  onGoLibrary: () => void;
}

const tabs: { id: AppTab; icon: typeof Monitor }[] = [
  { id: "library", icon: Monitor },
];

export function Sidebar({ active, onChange, selectedGame, onGoLibrary }: Props) {
  const { t } = useTranslation("sidebar");
  return (
    <aside className="surface-panel flex w-[252px] shrink-0 flex-col border-r">
      <div className="border-b border-[var(--color-border)] px-4 pb-4 pt-2">
        <div className="flex items-center gap-2.5">
          <img
            src="/logo.png"
            width={28}
            height={28}
            alt=""
            className="shrink-0 rounded-md"
          />
          <div className="min-w-0">
            <div className="truncate text-sm font-semibold text-[var(--color-text)]">
              Game Settings Master
            </div>
            <div className="mt-0.5 text-xs text-muted">UE · Unity · ReShade</div>
          </div>
        </div>
      </div>

      <nav className="flex-1 space-y-0.5 p-3">
        {tabs.map(({ id, icon: Icon }) => {
          const isActive = active === id;
          const label = t(`tabs.${id}.label`);
          const desc = t(`tabs.${id}.desc`);
          return (
            <button
              key={id}
              type="button"
              onClick={() => onChange(id)}
              className={cn(
                "flex w-full items-center gap-3 rounded-lg px-3 py-2.5 text-left transition",
                isActive
                  ? "bg-[var(--color-bg-active)] text-[var(--color-text)] ring-1 ring-[var(--color-accent)]/40"
                  : "text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-hover)] hover:text-[var(--color-text)]",
              )}
            >
              <span
                className={cn(
                  "flex h-8 w-8 items-center justify-center rounded-md",
                  isActive
                    ? "bg-[var(--color-accent-soft)] text-accent"
                    : "bg-[var(--color-bg-card)] text-muted",
                )}
              >
                <Icon size={17} />
              </span>
              <span>
                <span className="block text-sm font-medium">{label}</span>
                <span className="block text-xs text-muted">{desc}</span>
              </span>
            </button>
          );
        })}
      </nav>

      <div className="space-y-3 border-t border-[var(--color-border)] p-3">
        <LanguageToggle />
        {selectedGame ? (
          <button
            type="button"
            onClick={onGoLibrary}
            className="w-full overflow-hidden rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-card)] text-left transition hover:border-[var(--color-border-strong)] hover:bg-[var(--color-bg-hover)]"
          >
            <GameCover game={selectedGame} aspect="sidebar" className="rounded-none rounded-t-lg" />
            <div className="p-3">
              <div className="text-xs font-medium uppercase tracking-wide text-muted">
                {t("activeGame")}
              </div>
              <div className="mt-1 truncate text-sm font-semibold text-[var(--color-text)]">
                {selectedGame.name}
              </div>
              <div className="mt-2">
                {supportsIniPresets(selectedGame) ? (
                  <Badge tone="success">{t("badge.configOk")}</Badge>
                ) : supportsReShade(selectedGame) ? (
                  <Badge tone="accent">ReShade</Badge>
                ) : (
                  <Badge tone="warning">{t("badge.needInstallDir")}</Badge>
                )}
              </div>
            </div>
          </button>
        ) : (
          <div className="rounded-lg border border-dashed border-[var(--color-border)] p-3 text-center text-sm text-muted">
            {t("pickGame")}
          </div>
        )}
      </div>
    </aside>
  );
}
