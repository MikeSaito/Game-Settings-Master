import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Copy, Cpu, ExternalLink, FolderOpen, Play, X } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { useBackgroundSafeEnabled } from "@/hooks/app/useBackgroundSafeEnabled";
import { useGameRunning } from "@/hooks/game/useGameRunning";
import { closeGame, getGpuInfo, launchGame, openConfigFolder } from "@/lib/api";
import { formatInvokeError } from "@/lib/core";
import { exeNameForRunningCheck } from "@/lib/game";
import { supportsIniPresets } from "@/lib/game";
import { gpuSummaryLabel } from "@/lib/gpu";
import type { GameProfile } from "@/lib/core";
import { Badge } from "../ds/Badge";
import { Button } from "../ds/Button";
import { GameCover } from "@/components/game/GameCover";
interface Props {
  game: GameProfile;
}

export function GameContextBar({ game }: Props) {
  const { t } = useTranslation("header");
  const queryClient = useQueryClient();
  const gpuEnabled = useBackgroundSafeEnabled();
  const [copyStatus, setCopyStatus] = useState<"copied" | "failed" | null>(null);
  const [launchStatus, setLaunchStatus] = useState<{
    tone: "success" | "warning" | "danger";
    text: string;
  } | null>(null);
  const sessionRef = useRef(0);
  const configDir = game.config_dir;
  const runningExeName = exeNameForRunningCheck(game.exe_name, undefined);
  const gameRunning = useGameRunning(runningExeName);

  const { data: gpu } = useQuery({
    queryKey: ["gpu"],
    queryFn: getGpuInfo,
    enabled: gpuEnabled,
    staleTime: 300_000,
  });

  useEffect(() => {
    sessionRef.current += 1;
    setLaunchStatus(null);
  }, [game.id]);

  const launchMutation = useMutation({
    mutationFn: (session: number) => launchGame(game).then((result) => ({ result, session })),
    onSuccess: ({ result, session }) => {
      if (session !== sessionRef.current) return;
      setLaunchStatus({
        tone: result.warning ? "warning" : "success",
        text: result.warning ?? t("launchVia", { launcher: result.launcher }),
      });
      queryClient.invalidateQueries({ queryKey: ["game-running"] });
    },
    onError: (err, session) => {
      if (session !== sessionRef.current) return;
      setLaunchStatus({ tone: "danger", text: formatInvokeError(err) });
    },
  });

  const closeMutation = useMutation({
    mutationFn: (session: number) => {
      if (!runningExeName) return Promise.reject(new Error(t("errors.noProcessName")));
      return closeGame(game.id, runningExeName).then(() => session);
    },
    onSuccess: (session) => {
      if (session !== sessionRef.current) return;
      setLaunchStatus({ tone: "success", text: t("gameClosed") });
      queryClient.invalidateQueries({ queryKey: ["game-running"] });
    },
    onError: (err, session) => {
      if (session !== sessionRef.current) return;
      setLaunchStatus({ tone: "danger", text: formatInvokeError(err) });
    },
  });

  const handleCopyConfig = async () => {
    if (!configDir) return;
    try {
      await navigator.clipboard.writeText(configDir);
      setCopyStatus("copied");
    } catch {
      setCopyStatus("failed");
    }
  };

  return (
    <section className="relative z-40 shrink-0 border-b border-[var(--color-border)] bg-[var(--color-bg-soft)] px-4 py-3 touch-manipulation">      <div className="flex flex-wrap items-center gap-3">
        <GameCover game={game} aspect="square" className="h-12 w-12 shrink-0 rounded-[var(--radius-control)]" />
        <div className="min-w-0 flex-1">
          <div className="flex min-w-0 flex-wrap items-center gap-2">
            <h1 className="truncate text-base font-semibold text-[var(--color-text)]">{game.name}</h1>
            {game.is_ue ? (
              <Badge tone="accent">
                {game.engine_family === "ue4"
                  ? "UE 4"
                  : game.engine_family === "ue5"
                    ? "UE 5"
                    : "Unreal"}
              </Badge>
            ) : (
              <Badge tone="warning">{t("badge.engineUnknown")}</Badge>
            )}
            {game.engine_version && <Badge tone="neutral">{game.engine_version}</Badge>}
            <Badge tone={supportsIniPresets(game) ? "success" : "warning"}>
              {configDir ? t("badge.configOk") : t("badge.needConfig")}
            </Badge>
            {launchStatus && (
              <Badge tone={launchStatus.tone} className="max-w-[240px] truncate">
                {launchStatus.text}
              </Badge>
            )}
          </div>
        </div>
        <div className="ml-auto flex shrink-0 flex-wrap items-center gap-2">
          {gpu && (
            <Badge tone="info" title={gpuSummaryLabel(gpu)} className="max-w-[200px] truncate">
              <Cpu size={12} className="mr-1" />
              {gpuSummaryLabel(gpu)}
            </Badge>
          )}
          {gameRunning && runningExeName ? (
            <Button
              variant="secondary"
              icon={<X size={15} />}
              onClick={() => closeMutation.mutate(sessionRef.current)}
              loading={closeMutation.isPending}
            >
              {t("button.close")}
            </Button>
          ) : (
            <Button
              variant="primary"
              icon={<Play size={15} fill="currentColor" />}
              onClick={() => launchMutation.mutate(sessionRef.current)}
              loading={launchMutation.isPending}
            >
              {t("button.play")}
            </Button>
          )}
          {configDir && (
            <>
              <Button
                size="sm"
                variant="ghost"
                icon={<Copy size={14} />}
                onClick={() => void handleCopyConfig()}
              >
                {t("button.copy")}
              </Button>
              <Button
                size="sm"
                variant="secondary"
                icon={<FolderOpen size={14} />}
                onClick={() => openConfigFolder(configDir, game.id)}
              >
                {t("button.open")}
              </Button>
              {copyStatus && (
                <Badge tone={copyStatus === "copied" ? "success" : "warning"}>
                  {t(`button.${copyStatus}`)}
                </Badge>
              )}
            </>
          )}
          {!configDir && <ExternalLink size={16} className="text-[var(--color-text-faint)]" />}
        </div>
      </div>
      <div className="mt-2 flex min-w-0 items-center gap-2 pl-[3.75rem] text-xs text-[var(--color-text-muted)]">
        <span className="shrink-0 uppercase tracking-wide">{t("configLabel")}</span>
        <code className="truncate font-mono text-[var(--color-text-secondary)]">
          {configDir || t("noConfigPath")}
        </code>
      </div>
    </section>
  );
}