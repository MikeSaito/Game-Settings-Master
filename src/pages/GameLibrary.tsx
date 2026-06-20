import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { openPathDialog } from "../lib/tauriDialog";
import {
  FolderOpen,
  FolderSearch,
  Gamepad2,
  ImagePlus,
  Plus,
  RefreshCw,
  Search,
  Trash2,
  X,
} from "lucide-react";
import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { GameCover } from "../components/GameCover";
import { useBackgroundSafeEnabled } from "../hooks/useBackgroundSafeEnabled";
import {
  addManualGame,
  importGameCover,
  removeGameCover,
  removeGameProfile,
  scanGames,
  setGameConfigDir,
} from "../lib/api";
import { supportsIniPresets } from "../lib/gameEngine";
import type { GameProfile } from "../lib/types";
import { formatInvokeError } from "../lib/errors";
import { Alert } from "../components/ui/Alert";
import { Badge } from "../components/ui/Badge";
import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import { EmptyState } from "../components/ui/EmptyState";
import { Input } from "../components/ui/Input";
import { PageHeader } from "../components/ui/PageHeader";

interface Props {
  selectedGame: GameProfile | null;
  onSelectGame: (game: GameProfile) => void;
  onGameUpdated?: (game: GameProfile) => void;
  onGameRemoved?: (id: string) => void;
}

const sourceLabels: Record<string, string> = {
  steam: "Steam",
  epic: "Epic",
};

export function GameLibrary({ selectedGame, onSelectGame, onGameUpdated, onGameRemoved }: Props) {
  const { t } = useTranslation("library");
  const queryClient = useQueryClient();
  const [query, setQuery] = useState("");
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
      withConfig: games.filter((g) => g.config_dir).length,
      withoutConfig: games.filter((g) => !g.config_dir).length,
      ue: games.filter((g) => g.is_ue).length,
      unity: games.filter((g) => g.is_unity).length,
      other: games.filter((g) => !g.is_ue && !g.is_unity).length,
      total: games.length,
      withCover: games.filter((g) => g.custom_cover || g.cover_url).length,
    }),
    [games],
  );

  const { ueGames, unityGames, otherGames } = useMemo(() => {
    const q = query.toLowerCase();
    const matched = games.filter((g) => g.name.toLowerCase().includes(q));
    const byName = (a: GameProfile, b: GameProfile) =>
      a.name.toLowerCase().localeCompare(b.name.toLowerCase(), "ru");
    return {
      ueGames: matched.filter((g) => g.is_ue).sort(byName),
      unityGames: matched.filter((g) => g.is_unity).sort(byName),
      otherGames: matched.filter((g) => !g.is_ue && !g.is_unity).sort(byName),
    };
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
      if (profile) {
        queryClient.invalidateQueries({ queryKey: ["games"] });
        onSelectGame(profile);
        setQuery("");
      }
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
      if (profile && profile.id === gameId) {
        queryClient.invalidateQueries({ queryKey: ["games"] });
        onSelectGame(profile);
        onGameUpdated?.(profile);
      }
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
      if (profile && profile.id === gameId) {
        queryClient.invalidateQueries({ queryKey: ["games"] });
        onGameUpdated?.(profile);
      }
    },
    onError: (err) => setLibraryError(formatInvokeError(err)),
  });

  const removeCover = useMutation({
    mutationFn: (gameId: string) => removeGameCover(gameId),
    onMutate: () => setLibraryError(undefined),
    onSuccess: (profile, gameId) => {
      if (profile.id === gameId) {
        queryClient.invalidateQueries({ queryKey: ["games"] });
        onGameUpdated?.(profile);
      }
    },
    onError: (err) => setLibraryError(formatInvokeError(err)),
  });

  const renderGameCard = (game: GameProfile) => {
    const isSelected = selectedGame?.id === game.id;
    return (
      <Card
        key={game.id}
        selected={isSelected}
        padding="sm"
        className="group overflow-hidden !p-0"
      >
        <button
          type="button"
          onClick={() => onSelectGame(game)}
          className="block w-full text-left"
        >
          <GameCover
            game={game}
            aspect="header"
            selected={isSelected}
            className="rounded-none rounded-t-xl ring-0"
          />
          <div className="p-4">
            <div className="truncate font-semibold text-[var(--color-text)]">
              {game.name}
            </div>
            <div className="mt-2 flex flex-wrap gap-1.5">
              <Badge tone="default">
                {game.source === "manual"
                  ? t("source.manual")
                  : sourceLabels[game.source] ?? game.source}
              </Badge>
              {game.is_unity && (
                <Badge tone="accent">Unity</Badge>
              )}
              {game.is_ue && (
                <Badge tone="accent">
                  {game.engine_family === "ue4"
                    ? "UE 4"
                    : game.engine_family === "ue5"
                      ? "UE 5"
                      : "Unreal"}
                </Badge>
              )}
              {!game.is_ue && !game.is_unity && (
                <Badge tone="warning">{t("card.engineUnknown")}</Badge>
              )}
              {game.is_ue && game.possible_ue && (
                <Badge tone="default">{t("card.probablyUe")}</Badge>
              )}
              {game.is_unity && game.possible_unity && (
                <Badge tone="default">{t("card.probablyUnity")}</Badge>
              )}
              <Badge tone={game.config_dir ? "success" : "warning"}>
                {game.config_dir ? "Config OK" : "Config ?"}
              </Badge>
            </div>
            <p className="mt-2 truncate font-mono text-xs text-faint">
              {game.install_dir}
            </p>
          </div>
        </button>

        <div className="flex flex-wrap gap-2 border-t border-[var(--color-border)] px-4 py-3">
          {supportsIniPresets(game) && (
            <Button
              variant="ghost"
              className="!px-2 !py-1.5 text-xs text-accent"
              icon={<FolderOpen size={14} />}
              onClick={() => onSelectGame(game)}
            >
              {t("card.select")}
            </Button>
          )}
          {!game.config_dir && (
            <Button
              variant="secondary"
              className="!px-3 !py-1.5 text-xs"
              icon={<FolderSearch size={14} />}
              onClick={() => pickConfigDir.mutate(game.id)}
              loading={pickConfigDir.isPending}
            >
              Config
            </Button>
          )}
          <Button
            variant="secondary"
            className="!px-3 !py-1.5 text-xs"
            icon={<ImagePlus size={14} />}
            onClick={() => importCover.mutate(game.id)}
            loading={importCover.isPending}
          >
            {t("card.cover")}
          </Button>
          {game.custom_cover && (
            <Button
              variant="ghost"
              className="!px-2 !py-1.5 text-xs text-muted"
              icon={<X size={14} />}
              onClick={() => removeCover.mutate(game.id)}
              loading={removeCover.isPending}
            >
              {t("card.resetCover")}
            </Button>
          )}
          {game.source === "manual" && (
            <button
              type="button"
              onClick={() => removeGame.mutate(game.id)}
              className="ml-auto rounded-lg p-2 text-muted transition hover:bg-[#2e1a1a] hover:text-[#f08080]"
              title={t("card.removeProfile")}
            >
              <Trash2 size={16} />
            </button>
          )}
        </div>
      </Card>
    );
  };

  return (
    <div className="space-y-6">
      {libraryError && (
        <Alert tone="error" title={t("alerts.errorTitle")}>
          {libraryError}
        </Alert>
      )}
      <PageHeader
        title={t("header.title")}
        subtitle={t("header.subtitle")}
        meta={
          <>
            <Badge tone="success">{t("badges.withConfig", { count: scanSummary.withConfig })}</Badge>
            {scanSummary.withoutConfig > 0 && (
              <Badge tone="warning">{t("badges.withoutConfig", { count: scanSummary.withoutConfig })}</Badge>
            )}
            <Badge tone="default">{t("badges.ue", { count: scanSummary.ue })}</Badge>
            {scanSummary.unity > 0 && (
              <Badge tone="default">{t("badges.unity", { count: scanSummary.unity })}</Badge>
            )}
            {scanSummary.other > 0 && (
              <Badge tone="warning">{t("badges.other", { count: scanSummary.other })}</Badge>
            )}
            <Badge tone="info">{t("badges.withCover", { count: scanSummary.withCover })}</Badge>
          </>
        }
        actions={
          <Button
            variant="secondary"
            icon={<RefreshCw size={16} className={isFetching ? "animate-spin" : ""} />}
            onClick={() => refetch()}
            loading={isFetching}
          >
            {t("actions.scan")}
          </Button>
        }
      />

      <Alert tone="info" title={t("alerts.coversTitle")}>
        {t("alerts.coversBody")}
      </Alert>

      <div className="flex flex-wrap items-center gap-3">
        <div className="min-w-[260px] flex-1">
          <Input
            icon={<Search size={16} />}
            placeholder={t("search.placeholder")}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />
        </div>
        <Button
          variant="primary"
          icon={<Plus size={16} />}
          onClick={() => addManual.mutate()}
          loading={addManual.isPending}
        >
          {t("actions.add")}
        </Button>
      </div>

      {libraryLoading ? (
        <div className="flex flex-col items-center gap-4 py-20">
          <span className="h-10 w-10 animate-spin rounded-full border-2 border-[var(--color-border)] border-t-[var(--color-accent)]" />
          <p className="text-sm text-body">{t("loading")}</p>
        </div>
      ) : ueGames.length === 0 &&
        unityGames.length === 0 &&
        otherGames.length === 0 ? (
        <EmptyState
          icon={Gamepad2}
          title={query ? t("empty.nothingFound") : t("empty.noGames")}
          description={
            query ? (
              <>{t("empty.tryAnother")}</>
            ) : (
              <>
                {t("empty.scanOrAddPrefix")}
                <code className="text-code">
                  %LOCALAPPDATA%\Subnautica2\Saved\Config\Windows
                </code>
              </>
            )
          }
          action={
            <Button
              variant="primary"
              icon={<Plus size={16} />}
              onClick={() => addManual.mutate()}
              loading={addManual.isPending}
            >
              {t("actions.addManual")}
            </Button>
          }
        />
      ) : (
        <div className="space-y-8">
          {ueGames.length > 0 && (
            <section className="space-y-4">
              <div>
                <h2 className="text-lg font-semibold text-[var(--color-text)]">
                  Unreal Engine
                </h2>
              </div>
              <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
                {ueGames.map(renderGameCard)}
              </div>
            </section>
          )}

          {unityGames.length > 0 && (
            <section className="space-y-4 border-t border-[var(--color-border)] pt-8">
              <div>
                <h2 className="text-lg font-semibold text-[var(--color-text)]">
                  Unity
                </h2>
                <p className="mt-1 text-sm text-muted">
                  {t("sections.unityHint")}
                </p>
              </div>
              <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
                {unityGames.map(renderGameCard)}
              </div>
            </section>
          )}

          {otherGames.length > 0 && (
            <section className="space-y-4 border-t border-[var(--color-border)] pt-8">
              <div>
                <h2 className="text-lg font-semibold text-[var(--color-text)]">
                  {t("sections.otherTitle")}
                </h2>
                <p className="mt-1 text-sm text-muted">
                  {t("sections.otherHint")}
                </p>
              </div>
              <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
                {otherGames.map(renderGameCard)}
              </div>
            </section>
          )}
        </div>
      )}
    </div>
  );
}
