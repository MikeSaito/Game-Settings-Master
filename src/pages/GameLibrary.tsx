import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Gamepad2, Plus, RefreshCw } from "lucide-react";
import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { GameGridCard } from "../components/library/GameGridCard";
import { LibraryToolbar, type LibraryViewMode } from "../components/library/LibraryToolbar";
import { Alert, EmptyState, Skeleton } from "../components/ds/Feedback";
import { Badge } from "../components/ds/Badge";
import { Button } from "../components/ds/Button";
import { useBackgroundSafeEnabled } from "../hooks/useBackgroundSafeEnabled";
import {
  addManualGame,
  importGameCover,
  removeGameCover,
  removeGameProfile,
  scanGames,
  setGameConfigDir,
} from "../lib/api";
import { formatInvokeError } from "../lib/errors";
import { openPathDialog } from "../lib/tauriDialog";
import type { GameProfile } from "../lib/types";

interface Props {
  selectedGame: GameProfile | null;
  onSelectGame: (game: GameProfile) => void;
  onGameUpdated?: (game: GameProfile) => void;
  onGameRemoved?: (id: string) => void;
}

export function GameLibrary({ selectedGame, onSelectGame, onGameUpdated, onGameRemoved }: Props) {
  const { t } = useTranslation("library");
  const queryClient = useQueryClient();
  const [query, setQuery] = useState("");
  const [viewMode, setViewMode] = useState<LibraryViewMode>("grid");
  const [libraryError, setLibraryError] = useState<string>();
  const queriesEnabled = useBackgroundSafeEnabled();

  const { data: games = [], isLoading, isFetching, refetch } = useQuery({
    queryKey: ["games"],
    queryFn: scanGames,
    enabled: queriesEnabled,
    staleTime: 2 * 60_000,
    refetchOnMount: false,
  });

  const libraryLoading = (isLoading || isFetching) && games.length === 0;
  const scanSummary = useMemo(
    () => ({
      total: games.length,
      withConfig: games.filter((game) => game.config_dir).length,
      ue: games.filter((game) => game.is_ue).length,
      withCover: games.filter((game) => game.custom_cover || game.cover_url).length,
    }),
    [games],
  );

  const filteredGames = useMemo(() => {
    const q = query.trim().toLowerCase();
    return games
      .filter((game) => !q || game.name.toLowerCase().includes(q))
      .sort((a, b) => a.name.toLowerCase().localeCompare(b.name.toLowerCase(), "ru"));
  }, [games, query]);

  const addManual = useMutation({
    mutationFn: async () => {
      const path = await openPathDialog({
        directory: true,
        multiple: false,
        title: t("dialogs.installFolder"),
      });
      if (!path) return null;
      const name = query.trim() || path.split(/[/\\]/).pop() || "Custom Game";
      return addManualGame(name, path);
    },
    onMutate: () => setLibraryError(undefined),
    onSuccess: (profile) => {
      if (!profile) return;
      queryClient.invalidateQueries({ queryKey: ["games"] });
      onSelectGame(profile);
      setQuery("");
    },
    onError: (err) => setLibraryError(formatInvokeError(err)),
  });

  const pickConfigDir = useMutation({
    mutationFn: async (gameId: string) => {
      const path = await openPathDialog({
        directory: true,
        multiple: false,
        title: t("dialogs.configFolder"),
      });
      if (!path) return null;
      return setGameConfigDir(gameId, path);
    },
    onMutate: () => setLibraryError(undefined),
    onSuccess: (profile, gameId) => {
      if (!profile || profile.id !== gameId) return;
      queryClient.invalidateQueries({ queryKey: ["games"] });
      onSelectGame(profile);
      onGameUpdated?.(profile);
    },
    onError: (err) => setLibraryError(formatInvokeError(err)),
  });

  const removeGame = useMutation({
    mutationFn: (id: string) => removeGameProfile(id),
    onMutate: () => setLibraryError(undefined),
    onSuccess: (_data, id) => {
      queryClient.invalidateQueries({ queryKey: ["games"] });
      onGameRemoved?.(id);
    },
    onError: (err) => setLibraryError(formatInvokeError(err)),
  });

  const importCover = useMutation({
    mutationFn: async (gameId: string) => {
      const path = await openPathDialog({
        multiple: false,
        title: t("dialogs.coverFile"),
        filters: [
          {
            name: t("dialogs.imageFilter"),
            extensions: ["png", "jpg", "jpeg", "webp", "gif"],
          },
        ],
      });
      if (!path) return null;
      return importGameCover(gameId, path);
    },
    onMutate: () => setLibraryError(undefined),
    onSuccess: (profile, gameId) => {
      if (!profile || profile.id !== gameId) return;
      queryClient.invalidateQueries({ queryKey: ["games"] });
      onGameUpdated?.(profile);
    },
    onError: (err) => setLibraryError(formatInvokeError(err)),
  });

  const removeCover = useMutation({
    mutationFn: (gameId: string) => removeGameCover(gameId),
    onMutate: () => setLibraryError(undefined),
    onSuccess: (profile, gameId) => {
      if (profile.id !== gameId) return;
      queryClient.invalidateQueries({ queryKey: ["games"] });
      onGameUpdated?.(profile);
    },
    onError: (err) => setLibraryError(formatInvokeError(err)),
  });

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-end justify-between gap-3">
        <div>
          <p className="text-xs font-semibold uppercase tracking-[0.22em] text-[var(--color-accent)]">
            Game Settings Master
          </p>
          <h1 className="mt-1 text-2xl font-semibold tracking-tight text-[var(--color-text)]">
            {t("header.title")}
          </h1>
          <p className="mt-1 text-sm text-[var(--color-text-muted)]">{t("header.subtitle")}</p>
        </div>
        <div className="flex flex-wrap gap-1.5">
          <Badge tone="info">{t("badges.total", { count: scanSummary.total })}</Badge>
          <Badge tone="success">{t("badges.withConfig", { count: scanSummary.withConfig })}</Badge>
          <Badge tone="accent">{t("badges.ue", { count: scanSummary.ue })}</Badge>
          <Badge tone="neutral">{t("badges.withCover", { count: scanSummary.withCover })}</Badge>
        </div>
      </div>

      {libraryError && (
        <Alert tone="danger" title={t("alerts.errorTitle")}>
          {libraryError}
        </Alert>
      )}

      <LibraryToolbar
        query={query}
        onQueryChange={setQuery}
        viewMode={viewMode}
        onViewModeChange={setViewMode}
        onScan={() => void refetch()}
        scanning={isFetching}
        onAdd={() => addManual.mutate()}
        adding={addManual.isPending}
      />

      {libraryLoading ? (
        <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
          {Array.from({ length: 6 }, (_, index) => (
            <Skeleton key={index} className="h-56" />
          ))}
        </div>
      ) : filteredGames.length === 0 ? (
        <EmptyState
          icon={Gamepad2}
          title={query ? t("empty.nothingFound") : t("empty.noGames")}
          description={query ? t("empty.tryAnother") : t("empty.scanOrAddPrefix")}
          primaryAction={
            <Button
              variant="primary"
              icon={<RefreshCw size={15} />}
              onClick={() => void refetch()}
              loading={isFetching}
            >
              {t("actions.scan")}
            </Button>
          }
          secondaryAction={
            <Button
              variant="secondary"
              icon={<Plus size={15} />}
              onClick={() => addManual.mutate()}
              loading={addManual.isPending}
            >
              {t("actions.addManual")}
            </Button>
          }
        />
      ) : (
        <section
          className={
            viewMode === "grid"
              ? "grid gap-3 sm:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4"
              : "space-y-2"
          }
          aria-label={t("gamesLabel")}
        >
          {filteredGames.map((game) => (
            <GameGridCard
              key={game.id}
              game={game}
              selected={selectedGame?.id === game.id}
              mode={viewMode}
              onSelect={onSelectGame}
              onPickConfig={(id) => pickConfigDir.mutate(id)}
              pickingConfig={pickConfigDir.isPending}
              onImportCover={(id) => importCover.mutate(id)}
              importingCover={importCover.isPending}
              onRemoveCover={(id) => removeCover.mutate(id)}
              removingCover={removeCover.isPending}
              onRemoveGame={(id) => removeGame.mutate(id)}
            />
          ))}
        </section>
      )}
    </div>
  );
}
