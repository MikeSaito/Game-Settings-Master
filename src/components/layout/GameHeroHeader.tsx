import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  FolderOpen,
  History,
  Play,
  SlidersHorizontal,
  Sparkles,
  X,
} from "lucide-react";
import { useEffect, useState } from "react";
import { closeGame, getGpuInfo, launchGame, openConfigFolder } from "../../lib/api";
import { useGameRunning } from "../../hooks/useGameRunning";
import {
  gameCoverFallbackLetter,
  resolveGameHeroCoverCandidates,
} from "../../lib/gameCover";
import { isAuthorCuratedGame } from "../../lib/gameEngine";
import { gpuSummaryLabel } from "../../lib/gpuCompat";
import type { AppTab, GameProfile } from "../../lib/types";
import { cn } from "../../lib/cn";
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
  const gameRunning = useGameRunning(game.exe_name);

  const { data: gpu } = useQuery({
    queryKey: ["gpu"],
    queryFn: getGpuInfo,
    staleTime: 300_000,
  });

  const [launchMessage, setLaunchMessage] = useState<string>();
  const [launchError, setLaunchError] = useState<string>();

  const launchMutation = useMutation({
    mutationFn: () => launchGame(game),
    onSuccess: (result) => {
      setLaunchError(undefined);
      setLaunchMessage(`Запуск через ${result.launcher}…`);
    },
    onError: (err) => {
      setLaunchMessage(undefined);
      setLaunchError(String(err));
    },
  });

  const closeMutation = useMutation({
    mutationFn: () => closeGame(game.exe_name!),
    onSuccess: () => {
      setLaunchError(undefined);
      setLaunchMessage("Игра закрыта.");
      queryClient.invalidateQueries({ queryKey: ["game-running", game.exe_name] });
    },
    onError: (err) => {
      setLaunchMessage(undefined);
      setLaunchError(String(err));
    },
  });

  useEffect(() => {
    setCoverIndex(0);
    setLaunchMessage(undefined);
    setLaunchError(undefined);
  }, [game.id, game.custom_cover, game.cover_url]);

  const showCover = !!coverSrc;
  const configDir = game.config_dir;

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

        <div className="absolute bottom-4 right-8 z-10">
          {gameRunning && game.exe_name ? (
            <Button
              variant="secondary"
              icon={<X size={16} />}
              onClick={() => closeMutation.mutate()}
              loading={closeMutation.isPending}
              className="shadow-lg"
            >
              Закрыть
            </Button>
          ) : (
            <Button
              variant="primary"
              icon={<Play size={16} fill="currentColor" />}
              onClick={() => launchMutation.mutate()}
              loading={launchMutation.isPending}
              className="shadow-lg"
            >
              Играть
            </Button>
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

      {(launchMessage || launchError) && (
        <div className="border-b border-[var(--color-border)] bg-[var(--color-bg-elevated)]/80 px-8 py-2 backdrop-blur-sm">
          <div className="mx-auto max-w-6xl">
            {launchMessage && (
              <p className="text-sm text-[var(--color-accent)]">{launchMessage}</p>
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
              onClick={() => openConfigFolder(configDir)}
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
            {tabs.map(({ id, label, icon: Icon }) => {
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
    </header>
  );
}
