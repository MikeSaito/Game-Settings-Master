import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useCallback, useEffect, useMemo, useRef } from "react";
import { currentLanguage } from "@/i18n";
import { useAppWindowFocused } from "@/context/AppWindowFocusProvider";
import { useBackgroundSafeEnabled } from "@/hooks/app/useBackgroundSafeEnabled";
import { useGameRunning } from "@/hooks/game/useGameRunning";
import { useRunningExeName } from "@/hooks/game/useRunningExeName";
import {
  getGameOverrides,
  getGameParameters,
  getGpuInfo,
  getScalabilityLimits,
} from "@/lib/api";
import { normalizeParameterCategories } from "@/lib/editor";
import type { GameOverride, GameParameter, GameProfile } from "@/lib/core";

const FOCUS_DISK_REFRESH_MS = 60_000;
const EMPTY_PARAMETERS: GameParameter[] = [];
const EMPTY_OVERRIDES: GameOverride[] = [];

export function useEditorQueries(game: GameProfile | null) {
  const queryClient = useQueryClient();
  const configDir = game?.config_dir ?? "";
  const runningExeName = useRunningExeName(game);
  const gameRunning = useGameRunning(runningExeName);
  const queriesEnabled = useBackgroundSafeEnabled(!!configDir && !!game?.id);
  const overridesEnabled = useBackgroundSafeEnabled(!!game?.id);
  const gpuEnabled = useBackgroundSafeEnabled();
  const paramsDirtyRef = useRef(false);

  const { data: parameters = EMPTY_PARAMETERS, isLoading, isFetching } = useQuery({
    queryKey: [
      "parameters",
      configDir,
      game?.id,
      game?.engine_family,
      game?.engine_version,
      currentLanguage(),
    ],
    queryFn: () =>
      getGameParameters(
        configDir,
        game!.id,
        game!.install_dir,
        game!.engine_family,
        game!.engine_version,
      ),
    enabled: queriesEnabled,
    staleTime: 5 * 60_000,
    refetchOnMount: false,
    placeholderData: (previousData, previousQuery) =>
      previousQuery?.queryKey?.[2] === game?.id ? previousData : undefined,
  });

  const parametersLoading = (isLoading || isFetching) && parameters.length === 0;
  const normalizedParameters = useMemo(
    () => normalizeParameterCategories(parameters),
    [parameters],
  );

  const { data: limits } = useQuery({
    queryKey: ["limits", configDir, game?.install_dir, game?.id],
    queryFn: () => getScalabilityLimits(configDir, game!.id, game!.install_dir),
    enabled: queriesEnabled && !!game,
  });

  const { data: overrides = EMPTY_OVERRIDES } = useQuery({
    queryKey: ["overrides", game?.id],
    queryFn: () => getGameOverrides(game!.id),
    enabled: overridesEnabled,
  });

  const { data: gpu } = useQuery({
    queryKey: ["gpu"],
    queryFn: getGpuInfo,
    enabled: gpuEnabled,
    staleTime: 300_000,
  });

  const windowFocused = useAppWindowFocused();
  const lastDiskRefreshRef = useRef(0);

  const refreshFromDisk = useCallback(
    (force = false) => {
      if (!game?.id || !configDir) return;
      if (paramsDirtyRef.current) return;
      const now = Date.now();
      if (!force && now - lastDiskRefreshRef.current < FOCUS_DISK_REFRESH_MS) return;
      lastDiskRefreshRef.current = now;
      void queryClient.invalidateQueries({
        queryKey: ["parameters", configDir, game.id],
      });
      void queryClient.invalidateQueries({
        queryKey: ["limits", configDir, game.install_dir, game.id],
      });
      void queryClient.invalidateQueries({ queryKey: ["game-config"] });
    },
    [queryClient, configDir, game?.id, game?.install_dir],
  );

  const prevRunningRef = useRef(gameRunning);
  useEffect(() => {
    if (prevRunningRef.current && !gameRunning) {
      refreshFromDisk(true);
    }
    prevRunningRef.current = gameRunning;
  }, [gameRunning, refreshFromDisk]);

  const prevFocusedRef = useRef(windowFocused);
  useEffect(() => {
    if (!prevFocusedRef.current && windowFocused) {
      refreshFromDisk(false);
    }
    prevFocusedRef.current = windowFocused;
  }, [windowFocused, refreshFromDisk]);

  return {
    configDir,
    runningExeName,
    gameRunning,
    parameters,
    parametersLoading,
    normalizedParameters,
    limits,
    overrides,
    gpu,
    paramsDirtyRef,
  };
}
