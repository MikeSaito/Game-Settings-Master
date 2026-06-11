import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  AlertTriangle,
  FolderOpen,
  History,
  Palette,
  Play,
  SlidersHorizontal,
  Sparkles,
  X,
} from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { ReShadeApiPickerModal } from "../ReShadeApiPickerModal";
import { ReShadeDisclaimerModal } from "../ReShadeDisclaimerModal";
import {
  closeGame,
  getGpuInfo,
  getReShadeSettings,
  getReShadeStatus,
  getReShadeWorkspace,
  launchGame,
  openConfigFolder,
  setReShadeSettings,
} from "../../lib/api";
import { exeNameForRunningCheck } from "../../lib/gameRunning";
import { formatInvokeError } from "../../lib/errors";
import { useBackgroundSafeEnabled } from "../../hooks/useBackgroundSafeEnabled";
import { useGameRunning } from "../../hooks/useGameRunning";
import {
  gameCoverFallbackLetter,
  resolveGameHeroCoverCandidates,
} from "../../lib/gameCover";
import { isAuthorCuratedGame, resolveGameTab, supportsIniPresets, supportsReShade } from "../../lib/gameEngine";
import { gpuSummaryLabel } from "../../lib/gpuCompat";
import {
  blocksReShadeLaunch,
  buildPerGamePatch,
  isReShadeActiveForGame,
  isValidReShadeApi,
  savedGameApi,
  shouldPromptApi,
  suggestApiForGame,
} from "../../lib/reshade";
import type { AppTab, GameProfile, ReShadeSettings } from "../../lib/types";
import { cn } from "../../lib/cn";
import { Alert } from "../ui/Alert";
import { Badge } from "../ui/Badge";
import { Button } from "../ui/Button";

interface Props {
  game: GameProfile;
  activeTab: AppTab;
  onTabChange: (tab: AppTab) => void;
}

const tabs: {
  id: AppTab;
  label: string;
  icon: typeof Sparkles | typeof History | typeof SlidersHorizontal;
}[] = [
  { id: "wizard", label: "Авто пресеты", icon: Sparkles },
  { id: "advanced", label: "Ручной", icon: SlidersHorizontal },
  { id: "backups", label: "Бекапы", icon: History },
  { id: "reshade", label: "ReShade", icon: Palette },
];

const sourceLabel: Record<string, string> = {
  steam: "Steam",
  epic: "Epic",
  manual: "Вручную",
};

export function GameHeroHeader({ game, activeTab, onTabChange }: Props) {
  const queryClient = useQueryClient();
  const coverCandidates = resolveGameHeroCoverCandidates(game);
  const [coverIndex, setCoverIndex] = useState(0);
  const coverSrc = coverCandidates[coverIndex] ?? null;
  const gpuEnabled = useBackgroundSafeEnabled();

  const { data: gpu } = useQuery({
    queryKey: ["gpu"],
    queryFn: getGpuInfo,
    enabled: gpuEnabled,
    staleTime: 300_000,
  });

  const [launchMessage, setLaunchMessage] = useState<string>();
  const [launchWarning, setLaunchWarning] = useState<string>();
  const [launchError, setLaunchError] = useState<string>();
  const [launchDisclaimerOpen, setLaunchDisclaimerOpen] = useState(false);
  const [apiPickerOpen, setApiPickerOpen] = useState(false);
  const [playPreflightPending, setPlayPreflightPending] = useState(false);
  const [launchConfirmPending, setLaunchConfirmPending] = useState(false);

  const launchSessionRef = useRef(0);

  useEffect(() => {
    launchSessionRef.current += 1;
    setLaunchMessage(undefined);
    setLaunchWarning(undefined);
    setLaunchError(undefined);
    setLaunchDisclaimerOpen(false);
    setApiPickerOpen(false);
    setPlayPreflightPending(false);
    setLaunchConfirmPending(false);
  }, [game.id]);

  const reshadeSettingsQueryKey = ["reshade-settings", game.id] as const;
  const reshadeWorkspaceQueryKey = ["reshade-workspace", game.id] as const;

  const loadFreshReShadeWorkspace = async (opts?: { retainSettingsCache?: boolean }) => {
    if (!opts?.retainSettingsCache) {
      await queryClient.invalidateQueries({ queryKey: reshadeSettingsQueryKey });
    }
    await queryClient.invalidateQueries({ queryKey: reshadeWorkspaceQueryKey });
    const workspace = await queryClient.fetchQuery({
      queryKey: reshadeWorkspaceQueryKey,
      queryFn: () => getReShadeWorkspace(game),
    });
    queryClient.setQueryData(reshadeSettingsQueryKey, workspace.settings);
    return workspace;
  };

  const reshadeOk = supportsReShade(game);

  const { data: reshadeSettings } = useQuery({
    queryKey: reshadeSettingsQueryKey,
    queryFn: () => getReShadeSettings(game.id, game.engine_family),
    enabled: reshadeOk,
    staleTime: 30_000,
    retry: 1,
  });

  const { data: reshadeStatus } = useQuery({
    queryKey: ["reshade-status", game.id],
    queryFn: () => getReShadeStatus(game),
    enabled: reshadeOk,
    staleTime: 10_000,
    retry: 1,
  });

  const reshadeActiveForPrefetch = isReShadeActiveForGame(reshadeSettings?.settings, game.id);

  const { data: reshadeWorkspace } = useQuery({
    queryKey: reshadeWorkspaceQueryKey,
    queryFn: () => getReShadeWorkspace(game),
    enabled: reshadeOk && reshadeActiveForPrefetch,
    staleTime: 10_000,
    retry: 1,
  });

  const runningExeName = exeNameForRunningCheck(
    game.exe_name,
    reshadeWorkspace?.status.exe_path ?? reshadeStatus?.exe_path,
  );
  const gameRunning = useGameRunning(runningExeName);

  const syncReShadeSettingsCache = (updated: ReShadeSettings) => {
    queryClient.setQueryData(
      reshadeSettingsQueryKey,
      (old: Awaited<ReturnType<typeof getReShadeSettings>> | undefined) =>
        old ? { ...old, settings: updated } : old,
    );
  };

  const apis = reshadeWorkspace?.settings.apis ?? reshadeSettings?.apis ?? [];

  const launchMutation = useMutation({
    mutationFn: ({ skipReShade, session }: { skipReShade: boolean; session: number }) =>
      launchGame(game, skipReShade).then((result) => ({ result, session })),
    onSuccess: ({ result, session }) => {
      if (session !== launchSessionRef.current) return;
      setLaunchError(undefined);
      setLaunchMessage(`Запуск через ${result.launcher}…`);
      setLaunchWarning(result.warning);
      queryClient.invalidateQueries({ queryKey: ["game-running"] });
      queryClient.invalidateQueries({ queryKey: ["reshade-workspace", game.id] });
      queryClient.invalidateQueries({ queryKey: ["reshade-status", game.id] });
      queryClient.invalidateQueries({ queryKey: ["reshade-settings", game.id] });
    },
    onError: (err, { session }) => {
      if (session !== launchSessionRef.current) return;
      setLaunchMessage(undefined);
      setLaunchWarning(undefined);
      setLaunchError(formatInvokeError(err));
    },
  });

  const closeMutation = useMutation({
    mutationFn: (session: number) => {
      if (!runningExeName) {
        return Promise.reject(new Error("Имя процесса игры не определено"));
      }
      return closeGame(runningExeName).then(() => session);
    },
    onSuccess: (session) => {
      if (session !== launchSessionRef.current) return;
      setLaunchError(undefined);
      setLaunchMessage("Игра закрыта.");
      queryClient.invalidateQueries({ queryKey: ["game-running"] });
    },
    onError: (err, session) => {
      if (session !== launchSessionRef.current) return;
      setLaunchMessage(undefined);
      setLaunchError(formatInvokeError(err));
    },
  });

  const isStaleLaunchSession = (session: number) => session !== launchSessionRef.current;

  const runLaunch = async (skipReShade = false) => {
    const session = launchSessionRef.current;
    try {
      await launchMutation.mutateAsync({ skipReShade, session });
      if (isStaleLaunchSession(session)) return;
    } catch {
      // ошибка показана в launchMutation.onError
    }
  };

  const continueLaunchWithApi = async (api: string, remember: boolean) => {
    if (launchConfirmPending || launchMutation.isPending) return;
    const session = launchSessionRef.current;
    setLaunchConfirmPending(true);
    try {
      let workspace;
      try {
        workspace = await loadFreshReShadeWorkspace();
        if (isStaleLaunchSession(session)) return;
      } catch (err) {
        setLaunchError(
          formatInvokeError(
            err instanceof Error ? err : new Error("Не удалось загрузить настройки ReShade"),
          ),
        );
        setApiPickerOpen(false);
        return;
      }

      const block = blocksReShadeLaunch(workspace.status);
      if (block) {
        setLaunchError(block);
        setApiPickerOpen(false);
        return;
      }

      const settings = workspace.settings.settings;
      const next = buildPerGamePatch(settings, game.id, {
        api,
        api_remembered: remember,
      });

      const updated = await setReShadeSettings(next);
      if (isStaleLaunchSession(session)) return;
      syncReShadeSettingsCache(updated);
      setApiPickerOpen(false);
      await runLaunch(false);
    } catch (err) {
      setLaunchError(formatInvokeError(err));
      setApiPickerOpen(false);
    } finally {
      setLaunchConfirmPending(false);
    }
  };

  const handlePlayClick = async () => {
    if (playPreflightPending || launchMutation.isPending) return;
    const session = launchSessionRef.current;

    if (!reshadeOk) {
      void runLaunch();
      return;
    }

    setPlayPreflightPending(true);
    setLaunchError(undefined);
    try {
      let settingsResp;
      try {
        await queryClient.invalidateQueries({ queryKey: reshadeSettingsQueryKey });
        settingsResp = await queryClient.fetchQuery({
          queryKey: reshadeSettingsQueryKey,
          queryFn: () => getReShadeSettings(game.id, game.engine_family),
        });
        if (isStaleLaunchSession(session)) return;
      } catch (err) {
        setLaunchMessage(undefined);
        setLaunchError(
          formatInvokeError(
            err instanceof Error ? err : new Error("Не удалось загрузить настройки ReShade"),
          ),
        );
        return;
      }

      if (!isReShadeActiveForGame(settingsResp.settings, game.id)) {
        await runLaunch(false);
        return;
      }

      let workspace;
      try {
        workspace = await loadFreshReShadeWorkspace({ retainSettingsCache: true });
        if (isStaleLaunchSession(session)) return;
      } catch (err) {
        setLaunchMessage(undefined);
        setLaunchError(
          formatInvokeError(
            err instanceof Error ? err : new Error("Не удалось загрузить настройки ReShade"),
          ),
        );
        return;
      }

      const settings = workspace.settings.settings;
      const block = blocksReShadeLaunch(workspace.status);
      if (block) {
        setLaunchMessage(undefined);
        setLaunchError(block);
        return;
      }

      if (!settings.launch_warning_acknowledged) {
        setLaunchDisclaimerOpen(true);
        return;
      }

      if (shouldPromptApi(settings, game.id)) {
        setApiPickerOpen(true);
        return;
      }

      const api = savedGameApi(settings, game.id);
      if (!api || !isValidReShadeApi(api)) {
        setApiPickerOpen(true);
        return;
      }

      await runLaunch(false);
    } finally {
      setPlayPreflightPending(false);
    }
  };

  const confirmLaunchDisclaimer = async () => {
    if (launchConfirmPending || launchMutation.isPending) return;
    const session = launchSessionRef.current;
    setLaunchConfirmPending(true);
    try {
      let workspace;
      try {
        workspace = await loadFreshReShadeWorkspace();
        if (isStaleLaunchSession(session)) return;
      } catch (err) {
        setLaunchDisclaimerOpen(false);
        setLaunchError(
          formatInvokeError(
            err instanceof Error ? err : new Error("Не удалось загрузить настройки ReShade"),
          ),
        );
        return;
      }

      const block = blocksReShadeLaunch(workspace.status);
      if (block) {
        setLaunchDisclaimerOpen(false);
        setLaunchError(block);
        return;
      }

      const settings = workspace.settings.settings;
      const updated = await setReShadeSettings({
        ...settings,
        launch_warning_acknowledged: true,
      });
      if (isStaleLaunchSession(session)) return;
      syncReShadeSettingsCache(updated);
      setLaunchDisclaimerOpen(false);

      if (shouldPromptApi(updated, game.id) || !savedGameApi(updated, game.id)) {
        setApiPickerOpen(true);
        return;
      }

      await runLaunch(false);
    } catch (err) {
      setLaunchError(formatInvokeError(err));
      setLaunchDisclaimerOpen(false);
    } finally {
      setLaunchConfirmPending(false);
    }
  };

  useEffect(() => {
    setCoverIndex(0);
  }, [game.id, game.custom_cover, game.cover_url]);

  useEffect(() => {
    setLaunchMessage(undefined);
    setLaunchWarning(undefined);
    setLaunchError(undefined);
  }, [game.id]);

  const iniPresets = supportsIniPresets(game);

  const visibleTabs = tabs.filter(({ id }) => {
    if (id === "reshade") return reshadeOk;
    return iniPresets;
  });

  useEffect(() => {
    const tabAllowed =
      activeTab === "reshade"
        ? reshadeOk
        : activeTab === "wizard" || activeTab === "advanced" || activeTab === "backups"
          ? iniPresets
          : true;
    if (!tabAllowed) {
      onTabChange(resolveGameTab(game));
    }
  }, [game, activeTab, iniPresets, reshadeOk, onTabChange]);

  const showCover = !!coverSrc;
  const configDir = game.config_dir;
  const launchBusy = launchMutation.isPending || playPreflightPending || launchConfirmPending;

  return (
    <header>
      <div className="relative h-44 w-full overflow-hidden md:h-52 lg:h-60">
        {showCover ? (
          <img
            src={coverSrc}
            alt=""
            className="cover-crop-center"
            onError={() => {
              if (coverIndex + 1 < coverCandidates.length) {
                setCoverIndex((i) => i + 1);
              } else {
                setCoverIndex(coverCandidates.length);
              }
            }}
          />
        ) : (
          <div className="absolute inset-0 bg-gradient-to-br from-[#1a2438] via-[#151921] to-[#0f1115]" />
        )}

        <div
          className="pointer-events-none absolute inset-0 bg-gradient-to-t from-[var(--color-bg)] via-[var(--color-bg)]/75 to-transparent"
          aria-hidden
        />
        <div
          className="pointer-events-none absolute inset-x-0 bottom-0 h-2/3 bg-gradient-to-t from-[var(--color-bg)] to-transparent"
          aria-hidden
        />

        <div className="absolute bottom-4 right-8 z-10 flex flex-col items-end gap-2">
          {gameRunning && runningExeName ? (
            <Button
              variant="secondary"
              icon={<X size={16} />}
              onClick={() => closeMutation.mutate(launchSessionRef.current)}
              loading={closeMutation.isPending}
              className="shadow-lg"
            >
              Закрыть
            </Button>
          ) : (
            <>
              <Button
                variant="primary"
                icon={<Play size={16} fill="currentColor" />}
                onClick={() => void handlePlayClick()}
                loading={launchBusy}
                className="shadow-lg"
              >
                Играть
              </Button>
              {reshadeOk && (
                <button
                  type="button"
                  onClick={() => void runLaunch(true)}
                  disabled={launchBusy || gameRunning}
                  className="text-xs text-muted underline-offset-2 hover:text-[var(--color-text-secondary)] hover:underline disabled:opacity-50"
                >
                  Без ReShade
                </button>
              )}
            </>
          )}
        </div>

        <div className="absolute inset-x-0 bottom-0 px-8 pb-4 pt-16">
          <div className="mx-auto max-w-6xl">
            <h1 className="text-2xl font-bold tracking-tight text-[var(--color-text)] drop-shadow-sm md:text-3xl">
              {game.name}
            </h1>

            <div className="mt-3 flex flex-wrap items-center gap-2">
              {isAuthorCuratedGame(game) ? (
                <Badge tone="accent">От автора</Badge>
              ) : game.is_unity ? (
                <Badge tone="accent">Unity</Badge>
              ) : game.is_ue ? (
                game.engine_family === "ue4" ? (
                  <Badge tone="accent">UE 4</Badge>
                ) : game.engine_family === "ue5" ? (
                  <Badge tone="accent">UE 5</Badge>
                ) : (
                  <Badge tone="accent">Unreal Engine</Badge>
                )
              ) : (
                <Badge tone="warning">Движок ?</Badge>
              )}
              {game.engine_version && (
                <Badge tone="default">{game.engine_version}</Badge>
              )}
              <Badge tone={configDir ? "success" : "warning"}>
                {configDir ? "Config OK" : "Нужен config"}
              </Badge>
              <Badge tone="default">{sourceLabel[game.source] ?? game.source}</Badge>
              {gpu && <Badge tone="default">{gpuSummaryLabel(gpu)}</Badge>}
            </div>
          </div>
        </div>

        {!showCover && (
          <div className="pointer-events-none absolute right-8 top-6 hidden text-6xl font-bold text-white/10 md:block">
            {gameCoverFallbackLetter(game.name)}
          </div>
        )}
      </div>

      {(launchMessage || launchWarning || launchError) && (
        <div className="border-b border-[var(--color-border)] bg-[var(--color-bg-elevated)]/80 px-8 py-3 backdrop-blur-sm">
          <div className="mx-auto max-w-6xl space-y-2">
            {launchMessage && (
              <p className="text-sm text-[var(--color-accent)]">{launchMessage}</p>
            )}
            {launchWarning && (
              <Alert tone="warning" icon={AlertTriangle} title="ReShade">
                {launchWarning}
              </Alert>
            )}
            {launchError && <p className="text-sm text-[#f0a0a0]">{launchError}</p>}
          </div>
        </div>
      )}

      {configDir && (
        <div className="border-b border-[var(--color-border)] bg-[var(--color-bg-elevated)]/80 px-8 py-2.5 backdrop-blur-sm">
          <div className="mx-auto flex max-w-6xl items-center gap-3">
            <span className="shrink-0 text-xs font-medium uppercase tracking-wide text-muted">
              Config
            </span>
            <p className="min-w-0 flex-1 truncate font-mono text-xs text-[var(--color-text-secondary)]">
              {configDir}
            </p>
            <Button
              variant="ghost"
              icon={<FolderOpen size={15} />}
              onClick={() => openConfigFolder(configDir, game.id)}
              className="shrink-0 !px-2 !py-1.5 text-xs"
            >
              Открыть
            </Button>
          </div>
        </div>
      )}

      <div className="border-b border-[var(--color-border)] bg-[var(--color-bg-elevated)] px-8 py-2">
        <div className="mx-auto max-w-6xl">
          <nav
            className="flex flex-wrap gap-1 rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] p-1"
            aria-label="Разделы игры"
          >
            {visibleTabs.map(({ id, label, icon: Icon }) => {
              const isActive = activeTab === id;
              return (
                <button
                  key={id}
                  type="button"
                  onClick={() => onTabChange(id)}
                  className={cn(
                    "flex items-center gap-2 rounded-md px-4 py-2 text-sm font-medium transition",
                    isActive
                      ? "bg-[var(--color-bg-active)] text-[var(--color-text)] shadow-sm ring-1 ring-[var(--color-accent)]/35"
                      : "text-muted hover:bg-[var(--color-bg-hover)] hover:text-[var(--color-text-secondary)]",
                  )}
                >
                  <Icon size={16} className={isActive ? "text-accent" : undefined} />
                  {label}
                </button>
              );
            })}
          </nav>
        </div>
      </div>

      <ReShadeDisclaimerModal
        kind="launch"
        open={launchDisclaimerOpen}
        loading={launchBusy}
        onConfirm={() => void confirmLaunchDisclaimer()}
        onCancel={() => setLaunchDisclaimerOpen(false)}
      />

      <ReShadeApiPickerModal
        open={apiPickerOpen}
        apis={apis}
        initialApi={
          reshadeSettings?.settings
            ? savedGameApi(reshadeSettings.settings, game.id) ??
              suggestApiForGame(game, reshadeSettings.suggested_api)
            : suggestApiForGame(game, reshadeSettings?.suggested_api)
        }
        rememberDefault
        loading={launchBusy}
        title="Выберите API перед запуском"
        onConfirm={(api, remember) => void continueLaunchWithApi(api, remember)}
        onCancel={() => setApiPickerOpen(false)}
      />
    </header>
  );
}
