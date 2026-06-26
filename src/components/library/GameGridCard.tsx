import { FolderOpen, FolderSearch, ImagePlus, Trash2, X } from "lucide-react";
import { useTranslation } from "react-i18next";
import { supportsIniPresets } from "@/lib/game";
import type { GameProfile } from "@/lib/core";
import { cn } from "@/lib/core";
import { GameCover } from "@/components/game/GameCover";
import { Badge } from "@/components/ds/Badge";
import { Button } from "@/components/ds/Button";
import type { LibraryViewMode } from "./LibraryToolbar";

interface Props {
  game: GameProfile;
  selected: boolean;
  mode: LibraryViewMode;
  onSelect: (game: GameProfile) => void;
  onPickConfig: (gameId: string) => void;
  pickingConfig: boolean;
  onImportCover: (gameId: string) => void;
  importingCover: boolean;
  onRemoveCover: (gameId: string) => void;
  removingCover: boolean;
  onRemoveGame: (gameId: string) => void;
}

const sourceLabels: Record<string, string> = {
  steam: "Steam",
  epic: "Epic",
};

export function GameGridCard({
  game,
  selected,
  mode,
  onSelect,
  onPickConfig,
  pickingConfig,
  onImportCover,
  importingCover,
  onRemoveCover,
  removingCover,
  onRemoveGame,
}: Props) {
  const { t } = useTranslation("library");
  const canOpen = supportsIniPresets(game);
  const canPickConfig = !game.config_dir;
  const handlePrimaryAction = canOpen
    ? () => onSelect(game)
    : canPickConfig
      ? () => onPickConfig(game.id)
      : undefined;

  const chips = (
    <div className="flex flex-wrap gap-1.5">
      <Badge tone="neutral">
        {game.source === "manual" ? t("source.manual") : sourceLabels[game.source] ?? game.source}
      </Badge>
      {game.is_ue ? (
        <Badge tone="accent">
          {game.engine_family === "ue4" ? "UE 4" : game.engine_family === "ue5" ? "UE 5" : "Unreal"}
        </Badge>
      ) : (
        <Badge tone="warning">{t("card.engineUnknown")}</Badge>
      )}
      {game.is_ue && game.possible_ue && <Badge tone="info">{t("card.probablyUe")}</Badge>}
      <Badge tone={game.config_dir ? "success" : "warning"}>
        {game.config_dir ? t("card.configOk") : t("card.configMissing")}
      </Badge>
    </div>
  );

  const actions = (
    <div className="flex flex-wrap items-center gap-1.5">
      {canOpen ? (
        <Button size="sm" variant="primary" icon={<FolderOpen size={14} />} onClick={() => onSelect(game)}>
          {t("card.select")}
        </Button>
      ) : canPickConfig ? (
        <Button
          size="sm"
          variant="primary"
          icon={<FolderSearch size={14} />}
          onClick={() => onPickConfig(game.id)}
          loading={pickingConfig}
        >
          {t("card.pickConfig")}
        </Button>
      ) : null}
      <Button
        size="sm"
        variant="secondary"
        icon={<ImagePlus size={14} />}
        onClick={() => onImportCover(game.id)}
        loading={importingCover}
      >
        {t("card.cover")}
      </Button>
      {game.custom_cover && (
        <Button
          size="sm"
          variant="ghost"
          icon={<X size={14} />}
          onClick={() => onRemoveCover(game.id)}
          loading={removingCover}
        >
          {t("card.resetCover")}
        </Button>
      )}
      {game.source === "manual" && (
        <button
          type="button"
          onClick={() => onRemoveGame(game.id)}
          className="ml-auto rounded-[var(--radius-control)] p-2 text-[var(--color-text-muted)] transition hover:bg-[var(--color-danger-soft)] hover:text-[var(--color-danger)]"
          title={t("card.removeProfile")}
        >
          <Trash2 size={15} />
        </button>
      )}
    </div>
  );

  if (mode === "list") {
    return (
      <article
        className={cn(
          "grid grid-cols-[44px_minmax(0,1fr)] items-center gap-3 rounded-[var(--radius-panel)] border bg-[var(--color-surface)] p-2 transition hover:border-[var(--color-border-strong)] md:grid-cols-[44px_minmax(0,1fr)_auto]",
          selected ? "border-[var(--color-accent)]" : "border-[var(--color-border)]",
        )}
      >
        <button
          type="button"
          onClick={handlePrimaryAction}
          disabled={!handlePrimaryAction}
          className="text-left disabled:cursor-default"
        >
          <GameCover game={game} aspect="square" selected={selected} className="h-11 w-11" />
        </button>
        <button
          type="button"
          onClick={handlePrimaryAction}
          disabled={!handlePrimaryAction}
          className="min-w-0 text-left disabled:cursor-default"
        >
          <div className="truncate font-semibold text-[var(--color-text)]">{game.name}</div>
          <div className="mt-1 truncate font-mono text-xs text-[var(--color-text-faint)]">
            {game.install_dir}
          </div>
        </button>
        <div className="col-span-full flex min-w-0 flex-wrap items-center justify-start gap-3 md:col-auto md:justify-end">
          {chips}
          {actions}
        </div>
      </article>
    );
  }

  return (
    <article
      className={cn(
        "group overflow-hidden rounded-[var(--radius-panel)] border bg-[var(--color-surface)] transition hover:-translate-y-0.5 hover:border-[var(--color-border-strong)]",
        selected ? "border-[var(--color-accent)] shadow-[0_0_0_1px_var(--color-accent-ring)]" : "border-[var(--color-border)]",
      )}
    >
      <button
        type="button"
        onClick={handlePrimaryAction}
        disabled={!handlePrimaryAction}
        className="block w-full text-left disabled:cursor-default"
      >
        <GameCover game={game} aspect="header" selected={selected} className="rounded-none" />
        <div className="p-3">
          <div className="truncate font-semibold text-[var(--color-text)]">{game.name}</div>
          <div className="mt-2">{chips}</div>
          <p className="mt-2 truncate font-mono text-xs text-[var(--color-text-faint)]">
            {game.install_dir}
          </p>
        </div>
      </button>
      <div className="border-t border-[var(--color-border)] p-2 opacity-95 transition group-hover:opacity-100">
        {actions}
      </div>
    </article>
  );
}
