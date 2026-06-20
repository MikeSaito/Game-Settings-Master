import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { currentLanguage } from "../i18n";
import { useAppWindowFocused } from "../context/AppWindowFocusProvider";
import { useWorkspacePreset } from "../context/GameWorkspaceContext";
import { useBackgroundSafeEnabled } from "./useBackgroundSafeEnabled";
import { useGameRunning } from "./useGameRunning";
import { useRunningExeName } from "./useRunningExeName";
import {
  applyCustom,
  applyGameOverride,
  deleteGameOverride,
  getGameOverrides,
  getGameParameters,
  getGpuInfo,
  getScalabilityLimits,
  saveGameOverride,
} from "../lib/api";
import {
  buildCategoryList,
  countEngineStats,
  filterParamsByCategoryAndSearch,
} from "../lib/advancedEditorFilters";
import { isParamVisible } from "../lib/gpuCompat";
import { applyParamDependencies } from "../lib/paramDependencies";
import { buildCustomChanges } from "../lib/buildCustomChanges";
import {
  defaultValueFor,
  ENGINE_CATEGORIES,
  engineParamId,
  initialEngineEnabledKeys,
  resolveEditableCategories,
} from "../lib/engineParams";
import { formatInvokeError } from "../lib/errors";
import type { GameParameter, GameProfile } from "../lib/types";

const UNITY_EDITABLE = new Set([
  "Graphics",
  "Display",
  "API",
  "Jobs",
  "Startup",
  "System",
]);

const EDITABLE_FOR_APPLY = new Set([
  "Scalability",
  "Rendering",
  "Shadows",
  "Textures",
  "PostProcess",
  "Display",
  "GameSpecific",
  "Audio",
]);

export function useAdvancedEditorState(game: GameProfile | null) {
  const { t } = useTranslation("advanced");
  const queryClient = useQueryClient();
  const configDir = game?.config_dir ?? "";
  const isUnity = game?.is_unity || game?.engine_family === "unity";
  const runningExeName = useRunningExeName(game);
  const gameRunning = useGameRunning(runningExeName);
  const queriesEnabled = useBackgroundSafeEnabled(!!configDir && !!game?.id);
  const overridesEnabled = useBackgroundSafeEnabled(!!game?.id);
  const gpuEnabled = useBackgroundSafeEnabled();

  useWorkspacePreset(t("title"), "selected", !!configDir);
  const [params, setParams] = useState<GameParameter[]>([]);
  const paramsDirtyRef = useRef(false);
  const activeGameIdRef = useRef(game?.id);
  activeGameIdRef.current = game?.id;
  const [overrideName, setOverrideName] = useState(t("defaultPresetName"));
  const [message, setMessage] = useState<string>();
  const [applyError, setApplyError] = useState<string>();
  const [activeCategory, setActiveCategory] = useState<string>(
    isUnity ? "Graphics" : "Scalability",
  );
  const [search, setSearch] = useState("");
  const [engineEnabled, setEngineEnabled] = useState<Set<string>>(new Set());

  useEffect(() => {
    setMessage(undefined);
    setApplyError(undefined);
    paramsDirtyRef.current = false;
  }, [game?.id]);

  const { data: parameters = [], isLoading, isFetching } = useQuery({
    queryKey: ["parameters", configDir, game?.id, game?.engine_family, currentLanguage()],
    queryFn: () =>
      getGameParameters(
        configDir,
        game?.id,
        game?.install_dir,
        game?.engine_family,
      ),
    enabled: queriesEnabled,
    staleTime: 5 * 60_000,
    refetchOnMount: false,
    placeholderData: (previousData, previousQuery) =>
      previousQuery?.queryKey?.[2] === game?.id ? previousData : undefined,
  });

  const parametersLoading = (isLoading || isFetching) && parameters.length === 0;

  const { data: limits } = useQuery({
    queryKey: ["limits", configDir, game?.install_dir, game?.id],
    queryFn: () => getScalabilityLimits(configDir, game!.install_dir, game!.id),
    enabled: queriesEnabled && !!game,
  });

  const { data: overrides = [] } = useQuery({
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
  const refreshFromDisk = useCallback(() => {
    if (!game?.id || !configDir) return;
    if (paramsDirtyRef.current) return;
    void queryClient.invalidateQueries({
      queryKey: ["parameters", configDir, game.id],
    });
    void queryClient.invalidateQueries({
      queryKey: ["limits", configDir, game.install_dir, game.id],
    });
    void queryClient.invalidateQueries({ queryKey: ["game-config"] });
  }, [queryClient, configDir, game?.id, game?.install_dir]);

  const prevRunningRef = useRef(gameRunning);
  useEffect(() => {
    if (prevRunningRef.current && !gameRunning) {
      refreshFromDisk();
    }
    prevRunningRef.current = gameRunning;
  }, [gameRunning, refreshFromDisk]);

  const prevFocusedRef = useRef(windowFocused);
  useEffect(() => {
    if (!prevFocusedRef.current && windowFocused) {
      refreshFromDisk();
    }
    prevFocusedRef.current = windowFocused;
  }, [windowFocused, refreshFromDisk]);

  const visibleParams = useMemo(
    () => params.filter((p) => isParamVisible(p, gpu)),
    [params, gpu],
  );

  useEffect(() => {
    if (paramsDirtyRef.current) return;
    setParams(parameters);
    setEngineEnabled(initialEngineEnabledKeys(parameters));
    if (isUnity) {
      setActiveCategory("Graphics");
    }
  }, [parameters, isUnity]);

  const categories = useMemo(
    () => buildCategoryList(visibleParams),
    [visibleParams],
  );

  useEffect(() => {
    if (categories.length && !categories.some((c) => c.cat === activeCategory)) {
      setActiveCategory(categories[0].cat);
    }
  }, [categories, activeCategory]);

  const filteredParams = useMemo(
    () =>
      filterParamsByCategoryAndSearch(
        visibleParams,
        activeCategory,
        search,
        engineEnabled,
      ),
    [visibleParams, activeCategory, search, engineEnabled],
  );

  const engineStats = useMemo(
    () => countEngineStats(visibleParams, engineEnabled),
    [visibleParams, engineEnabled],
  );

  const catalogStats = useMemo(() => {
    const known = visibleParams.filter((p) => p.known).length;
    return {
      known,
      unknown: visibleParams.length - known,
      total: visibleParams.length,
    };
  }, [visibleParams]);

  const editableCategories = useMemo(() => {
    const base = isUnity ? UNITY_EDITABLE : EDITABLE_FOR_APPLY;
    return resolveEditableCategories(parameters, base);
  }, [parameters, isUnity]);

  const buildChanges = () =>
    buildCustomChanges(params, parameters, gpu, engineEnabled, editableCategories);

  const updateParam = (key: string, section: string, file: string, value: string) => {
    paramsDirtyRef.current = true;
    setParams((prev) =>
      applyParamDependencies(prev, { key, section, file, value }, gpu),
    );
  };

  const toggleEngineParam = (p: GameParameter, enabled: boolean) => {
    paramsDirtyRef.current = true;
    const id = engineParamId(p);
    setEngineEnabled((prev) => {
      const next = new Set(prev);
      if (enabled) next.add(id);
      else next.delete(id);
      return next;
    });
    if (enabled && !p.value.trim()) {
      updateParam(p.key, p.section, p.file, defaultValueFor(p));
    }
  };

  const applyCustomMutation = useMutation({
    mutationFn: async () => {
      const snapshot = { gameId: game!.id, configDir };
      const { files, removals } = buildChanges();
      if (
        Object.keys(files).length === 0 &&
        Object.keys(removals).length === 0
      ) {
        throw new Error(
          isUnity ? t("errors.noChangesUnity") : t("errors.noChanges"),
        );
      }
      const result = await applyCustom(
        snapshot.configDir,
        files,
        runningExeName ?? undefined,
        removals,
        snapshot.gameId,
        game?.engine_family,
      );
      return { result, snapshot };
    },
    onMutate: () => setApplyError(undefined),
    onSuccess: ({ result, snapshot }) => {
      if (activeGameIdRef.current !== snapshot.gameId) return;
      paramsDirtyRef.current = false;
      setMessage(
        t("applied", {
          count: result.diff.length,
          backupId: result.backup_id,
        }),
      );
      queryClient.invalidateQueries({
        queryKey: ["backups", snapshot.configDir, snapshot.gameId],
      });
      queryClient.invalidateQueries({
        queryKey: ["parameters", snapshot.configDir, snapshot.gameId],
      });
      queryClient.invalidateQueries({ queryKey: ["game-config"] });
    },
    onError: (err) => setApplyError(formatInvokeError(err)),
  });

  const saveOverrideMutation = useMutation({
    mutationFn: async () => {
      const snapshot = { gameId: game!.id, name: overrideName };
      const { files, removals } = buildChanges();
      await saveGameOverride({
        game_id: snapshot.gameId,
        name: snapshot.name,
        files,
        removals,
      });
      return snapshot;
    },
    onSuccess: (snapshot) => {
      if (activeGameIdRef.current !== snapshot.gameId) return;
      queryClient.invalidateQueries({ queryKey: ["overrides", snapshot.gameId] });
      setMessage(t("presetSaved", { name: snapshot.name }));
    },
    onError: (err) => setApplyError(formatInvokeError(err)),
  });

  const applyOverrideMutation = useMutation({
    mutationFn: async (override: (typeof overrides)[0]) => {
      const snapshot = { gameId: game!.id, configDir };
      const result = await applyGameOverride(
        snapshot.configDir,
        override,
        runningExeName ?? undefined,
      );
      return { result, snapshot };
    },
    onSuccess: ({ result, snapshot }) => {
      if (activeGameIdRef.current !== snapshot.gameId) return;
      setMessage(t("presetApplied", { backupId: result.backup_id }));
      queryClient.invalidateQueries({
        queryKey: ["backups", snapshot.configDir, snapshot.gameId],
      });
      queryClient.invalidateQueries({
        queryKey: ["parameters", snapshot.configDir, snapshot.gameId],
      });
      queryClient.invalidateQueries({ queryKey: ["game-config"] });
    },
    onError: (err) => setApplyError(formatInvokeError(err)),
  });

  const deleteOverrideMutation = useMutation({
    mutationFn: ({ gameId, name }: { gameId: string; name: string }) =>
      deleteGameOverride(gameId, name),
    onSuccess: (_result, variables) => {
      if (activeGameIdRef.current !== variables.gameId) return;
      queryClient.invalidateQueries({ queryKey: ["overrides", variables.gameId] });
    },
    onError: (err) => setApplyError(formatInvokeError(err)),
  });

  return {
    game,
    configDir,
    isUnity,
    runningExeName,
    gameRunning,
    gpu,
    limits,
    overrides,
    message,
    applyError,
    activeCategory,
    setActiveCategory,
    search,
    setSearch,
    engineEnabled,
    categories,
    filteredParams,
    engineStats,
    catalogStats,
    parametersLoading,
    overrideName,
    setOverrideName,
    updateParam,
    toggleEngineParam,
    applyCustomMutation,
    saveOverrideMutation,
    applyOverrideMutation,
    deleteOverrideMutation,
    showEngineIniHint: ENGINE_CATEGORIES.has(activeCategory),
  };
}

export type AdvancedEditorState = ReturnType<typeof useAdvancedEditorState>;
