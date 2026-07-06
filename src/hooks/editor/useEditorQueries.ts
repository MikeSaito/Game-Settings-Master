import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useCallback, useEffect, useMemo, useRef } from "react";
import { currentLanguage } from "@/i18n";
import { useAppWindowFocused } from "@/context/AppWindowFocusProvider";
import { useBackgroundSafeEnabled } from "@/hooks/app/useBackgroundSafeEnabled";
import { useGameRunning } from "@/hooks/game/useGameRunning";
import { useRunningExeName } from "@/hooks/game/useRunningExeName";
import {
  getGameConfig,
  getGameOverrides,
  getGameParameters,
  getGpuInfo,
  getScalabilityLimits,
} from "@/lib/api";
import { normalizeParameterCategories } from "@/lib/editor";
import { EXTRA_INI_FILES } from "@/lib/ini/configFiles";
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

  const { data: limits, isLoading: limitsIsLoading, isFetching: limitsIsFetching } = useQuery({
    queryKey: ["limits", configDir, game?.install_dir, game?.id],
    queryFn: () => getScalabilityLimits(configDir, game!.id, game!.install_dir),
    enabled: queriesEnabled && !!game,
  });

  const limitsLoading = (limitsIsLoading || limitsIsFetching) && limits === undefined;

  const { data: overrides = EMPTY_OVERRIDES } = useQuery({
    queryKey: ["overrides", game?.id],
    queryFn: () => getGameOverrides(game!.id),
    enabled: overridesEnabled,
  });

  const { data: gpu, isLoading: gpuIsLoading, isFetching: gpuIsFetching, isFetched: gpuIsFetched } = useQuery({
    queryKey: ["gpu"],
    queryFn: getGpuInfo,
    enabled: gpuEnabled,
    staleTime: 300_000,
  });

  const gpuLoading = gpuEnabled && !gpuIsFetched && (gpuIsLoading || gpuIsFetching);
  const gpuUnavailable = gpuEnabled && gpuIsFetched && gpu === undefined;

  const { data: gameConfig, isLoading: gameConfigIsLoading, isFetching: gameConfigIsFetching, isFetched: gameConfigIsFetched } = useQuery({
    queryKey: ["game-config", configDir, game?.id, game?.engine_family],
    queryFn: () => getGameConfig(configDir, game!.id, game!.engine_family),
    enabled: queriesEnabled,
    staleTime: 5 * 60_000,
    refetchOnMount: false,
  });

  const gameConfigLoading =
    queriesEnabled && !gameConfigIsFetched && (gameConfigIsLoading || gameConfigIsFetching);

  const extraIniAvailable = useMemo(
    () => !!gameConfig?.files && EXTRA_INI_FILES.some((file) => file in gameConfig.files),
    [gameConfig],
  );

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
    limitsLoading,
    overrides,
    gpu,
    gpuLoading,
    gpuUnavailable,
    gameConfig,
    gameConfigLoading,
    gameConfigFetched: gameConfigIsFetched,
    extraIniAvailable,
    paramsDirtyRef,
  };
}
