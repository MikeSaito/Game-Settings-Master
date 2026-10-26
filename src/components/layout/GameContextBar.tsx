import { Copy, FolderOpen, Info, Play, X } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useQuery } from "@tanstack/react-query";
import { useBackgroundSafeEnabled } from "@/hooks/app/useBackgroundSafeEnabled";
import { useGameLaunch } from "@/hooks/game/useGameLaunch";
import { useGameRunning } from "@/hooks/game/useGameRunning";
import { getGpuInfo, openConfigFolder } from "@/lib/api";
import { exeNameForRunningCheck } from "@/lib/game";
import { DetailDialog } from "@/components/ds/DetailDialog";
import { GameHardwarePanel } from "@/components/advanced/GameHardwarePanel";
import { DiagnosticsPanel } from "@/components/advanced/DiagnosticsPanel";
import type { GameProfile } from "@/lib/core";
import { Badge, Button } from "@/components/ds";
import { GameCover } from "@/components/game/GameCover";
import { GameLaunchOptions } from "@/components/game/GameLaunchOptions";

interface Props {
  game: GameProfile;
}

export function GameContextBar({ game }: Props) {
  const { t } = useTranslation("header");
  const gpuEnabled = useBackgroundSafeEnabled();
  const [copyStatus, setCopyStatus] = useState<"copied" | "failed" | null>(
    null,
  );
  const [infoOpen, setInfoOpen] = useState(false);
  const configDir = game.config_dir;
  const runningExeName =
    exeNameForRunningCheck(game.exe_name, undefined) ?? null;
  const gameRunning = useGameRunning(runningExeName);
  const { launchStatus, launchMutation, closeMutation, sessionRef } =
    useGameLaunch(game, runningExeName);

  const { data: gpu } = useQuery({
    queryKey: ["gpu", game.id],
    queryFn: () => getGpuInfo(game.id),
    enabled: gpuEnabled,
    staleTime: 300_000,
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
    <section className="relative z-40 shrink-0 px-6 py-4 max-sm:px-3">
      <div className="flex items-center gap-3">
        <GameCover
          game={game}
          aspect="square"
          className="h-12 w-12 shrink-0 rounded-2xl"
        />
        <div className="min-w-0 flex-1">
          <h1 className="truncate text-lg font-medium">{game.name}</h1>
          <p className="truncate text-xs text-[var(--color-text-muted)]">
            {game.source === "manual"
              ? t("manualSource")
              : game.source.toUpperCase()}{" "}
            ·{" "}
            {game.engine_version ??
              (game.is_ue ? "Unreal Engine" : t("badge.engineUnknown"))}
          </p>
        </div>
        {launchStatus && (
          <Badge tone={launchStatus.tone} className="max-w-[200px] truncate">
            {launchStatus.text}
          </Badge>
        )}
        <button
          type="button"
          className="material-icon-button"
          aria-label={t("gameDetails")}
          title={t("gameDetails")}
          onClick={() => setInfoOpen(true)}
        >
          <Info size={21} />
        </button>
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
            variant="secondary"
            icon={<Play size={15} fill="currentColor" />}
            onClick={() => launchMutation.mutate(sessionRef.current)}
            loading={launchMutation.isPending}
          >
            {t("button.play")}
          </Button>
        )}
      </div>
      <DetailDialog
        key={game.id}
        open={infoOpen}
        title={t("gameDetails")}
        onClose={() => setInfoOpen(false)}
      >
        <div className="space-y-4">
          <h3 className="font-medium">{game.name}</h3>
          <div className="flex flex-wrap gap-2">
            <Badge tone={configDir ? "success" : "warning"}>
              {configDir ? t("badge.configOk") : t("badge.needConfig")}
            </Badge>
            {game.engine_version && <Badge>{game.engine_version}</Badge>}
          </div>
          <p className="text-sm text-[var(--color-text-muted)]">
            {t("configLabel")}
          </p>
          <code className="block break-all text-sm">
            {configDir || t("noConfigPath")}
          </code>
          {configDir && (
            <div className="flex flex-wrap items-center gap-2">
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
                icon={<FolderOpen size={14} />}
                onClick={() => openConfigFolder(configDir, game.id)}
              >
                {t("button.open")}
              </Button>
              {copyStatus && (
                <span role="status" className="text-sm">
                  {t(`button.${copyStatus}`)}
                </span>
              )}
            </div>
          )}
          <GameHardwarePanel game={game} gpu={gpu} />
          <DiagnosticsPanel game={game} />
          {(game.source === "manual" ||
            game.source === "gog" ||
            game.source === "xbox") && (
            <GameLaunchOptions key={game.id} game={game} />
          )}
        </div>
      </DetailDialog>
    </section>
  );
}
