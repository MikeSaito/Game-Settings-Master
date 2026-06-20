import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  AlertTriangle,
  FolderOpen,
  History,
  Play,
  SlidersHorizontal,
  X,
} from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Link, useNavigate } from "react-router-dom";
import { closeGame, getGpuInfo, launchGame, openConfigFolder } from "../../lib/api";
import { exeNameForRunningCheck } from "../../lib/gameRunning";
import { formatInvokeError } from "../../lib/errors";
import { useBackgroundSafeEnabled } from "../../hooks/useBackgroundSafeEnabled";
import { useGameRunning } from "../../hooks/useGameRunning";
import {
  gameCoverFallbackLetter,
  resolveGameHeroCoverCandidates,
} from "../../lib/gameCover";
import { resolveGameTabRoute, supportsIniPresets } from "../../lib/gameEngine";
import { gameTabPath } from "../../lib/routes";
import { gpuSummaryLabel } from "../../lib/gpuCompat";
import type { AppTab, GameProfile, GameTabRoute } from "../../lib/types";
import { cn } from "../../lib/cn";
import { Alert } from "../ui/Alert";
import { Badge } from "../ui/Badge";
import { Button } from "../ui/Button";

interface Props {
  game: GameProfile;
  activeTab: AppTab;
}

const tabs: {
  id: GameTabRoute;
  icon: typeof History | typeof SlidersHorizontal;
}[] = [
  { id: "advanced", icon: SlidersHorizontal },
  { id: "backups", icon: History },
];

export function GameHeroHeader({ game, activeTab }: Props) {
  const { t } = useTranslation("header");
  const navigate = useNavigate();
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

  const launchSessionRef = useRef(0);

  useEffect(() => {
    launchSessionRef.current += 1;
    setLaunchMessage(undefined);
    setLaunchWarning(undefined);
    setLaunchError(undefined);
  }, [game.id]);

  const runningExeName = exeNameForRunningCheck(game.exe_name, undefined);
  const gameRunning = useGameRunning(runningExeName);

  const launchMutation = useMutation({
    mutationFn: (session: number) =>
      launchGame(game).then((result) => ({ result, session })),
    onSuccess: ({ result, session }) => {
      if (session !== launchSessionRef.current) return;
      setLaunchError(undefined);
      setLaunchMessage(t("launchVia", { launcher: result.launcher }));
      setLaunchWarning(result.warning ?? undefined);
      queryClient.invalidateQueries({ queryKey: ["game-running"] });
    },
    onError: (err, session) => {
      if (session !== launchSessionRef.current) return;
      setLaunchMessage(undefined);
      setLaunchWarning(undefined);
      setLaunchError(formatInvokeError(err));
    },
  });

  const closeMutation = useMutation({
    mutationFn: (session: number) => {
      if (!runningExeName) {
        return Promise.reject(new Error(t("errors.noProcessName")));
      }
      return closeGame(runningExeName).then(() => session);
    },
    onSuccess: (session) => {
      if (session !== launchSessionRef.current) return;
      setLaunchError(undefined);
      setLaunchMessage(t("gameClosed"));
      queryClient.invalidateQueries({ queryKey: ["game-running"] });
    },
    onError: (err, session) => {
      if (session !== launchSessionRef.current) return;
      setLaunchMessage(undefined);
      setLaunchError(formatInvokeError(err));
    },
  });

  const handlePlayClick = () => {
    if (launchMutation.isPending) return;
    setLaunchError(undefined);
    setLaunchMessage(undefined);
    setLaunchWarning(undefined);
    launchMutation.mutate(launchSessionRef.current);
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

  const visibleTabs = tabs.filter(() => iniPresets);

  useEffect(() => {
    const tabAllowed =
      activeTab === "advanced" || activeTab === "backups" ? iniPresets : true;
    if (!tabAllowed) {
      const fallback = resolveGameTabRoute(game);
      if (fallback) {
        navigate(gameTabPath(game.id, fallback), { replace: true });
      }
    }
  }, [game, activeTab, iniPresets, navigate]);

  const showCover = !!coverSrc;
  const configDir = game.config_dir;
  const launchBusy = launchMutation.isPending;

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
              {t("button.close")}
            </Button>
          ) : (
            <Button
              variant="primary"
              icon={<Play size={16} fill="currentColor" />}
              onClick={handlePlayClick}
              loading={launchBusy}
              className="shadow-lg"
            >
              {t("button.play")}
            </Button>
          )}
        </div>

        <div className="absolute inset-x-0 bottom-0 px-8 pb-4 pt-16">
          <div className="mx-auto max-w-6xl">
            <h1 className="text-2xl font-bold tracking-tight text-[var(--color-text)] drop-shadow-sm md:text-3xl">
              {game.name}
            </h1>

            <div className="mt-3 flex flex-wrap items-center gap-2">
              {game.is_unity ? (
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
                <Badge tone="warning">{t("badge.engineUnknown")}</Badge>
              )}
              {game.engine_version && (
                <Badge tone="default">{game.engine_version}</Badge>
              )}
              <Badge tone={configDir ? "success" : "warning"}>
                {configDir ? t("badge.configOk") : t("badge.needConfig")}
              </Badge>
              <Badge tone="default">
                {t(`source.${game.source}`, { defaultValue: game.source })}
              </Badge>
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
              <Alert tone="warning" icon={AlertTriangle} title={t("launchWarningTitle")}>
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
              {t("configLabel")}
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
              {t("button.open")}
            </Button>
          </div>
        </div>
      )}

      {visibleTabs.length > 0 && (
        <div className="border-b border-[var(--color-border)] bg-[var(--color-bg-elevated)] px-8 py-2">
          <div className="mx-auto max-w-6xl">
            <nav
              className="flex flex-wrap gap-1 rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] p-1"
              aria-label={t("gameSections")}
            >
              {visibleTabs.map(({ id, icon: Icon }) => {
                const isActive = activeTab === id;
                return (
                  <Link
                    key={id}
                    to={gameTabPath(game.id, id)}
                    className={cn(
                      "flex items-center gap-2 rounded-md px-4 py-2 text-sm font-medium transition",
                      isActive
                        ? "bg-[var(--color-bg-active)] text-[var(--color-text)] shadow-sm ring-1 ring-[var(--color-accent)]/35"
                        : "text-muted hover:bg-[var(--color-bg-hover)] hover:text-[var(--color-text-secondary)]",
                    )}
                  >
                    <Icon size={16} className={isActive ? "text-accent" : undefined} />
                    {t(`tabs.${id}`)}
                  </Link>
                );
              })}
            </nav>
          </div>
        </div>
      )}
    </header>
  );
}
