import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useCallback, useEffect, useMemo, useRef, useState, useDeferredValue } from "react";
import { useTranslation } from "react-i18next";
import { currentLanguage } from "@/i18n";
import { useAppWindowFocused } from "@/context/AppWindowFocusProvider";
import { useWorkspacePreset } from "@/context/GameWorkspaceContext";
import { useBackgroundSafeEnabled } from "@/hooks/app/useBackgroundSafeEnabled";
import { useAppSettings } from "@/hooks/app/useAppSettings";
import { useGameRunning } from "@/hooks/game/useGameRunning";
import { useRunningExeName } from "@/hooks/game/useRunningExeName";
import { countPendingChanges } from "@/hooks/editor/editorStateUtils";
import { useEditorMutations } from "@/hooks/editor/useEditorMutations";
import { useEditorPanelState } from "@/hooks/editor/useEditorPanelState";
import {
  getGameOverrides,
  getGameParameters,
  getGpuInfo,
  getScalabilityLimits,
} from "@/lib/api";
import {
  ALL_CATEGORY,
  ENGINE_CATEGORIES,
  applyParamDependencies,
  buildCategoryList,
  buildCustomChanges,
  collectPendingKeys,
  countEngineStats,
  defaultValueFor,
  detectSgEngineConflicts,
  engineParamId,
  filterParamsByCategory,
  filterParamsBySearch,
  initialEngineEnabledKeys,
  normalizeParameterCategories,
  resolveEditableCategories,
  shouldSortByEngineEnabled,
  sortParamsForEngineCategory,
} from "@/lib/editor";
import { filterParamsByMode, filterParamsByPanel } from "@/lib/routing";
import { isParamVisible } from "@/lib/gpu";
import type { GameParameter, GameProfile } from "@/lib/core";

const EDITABLE_FOR_APPLY = new Set([
  "Scalability",
  "Rendering",
  "Shadows",
  "Textures",
  "PostProcess",
  "Display",
  "Window",
  "GameSpecific",
  "Audio",
  "Performance",
  "Other",
]);

const EMPTY_ENGINE_ENABLED = new Set<string>();

export function useAdvancedEditorState(game: GameProfile | null) {
  const { t } = useTranslation("advanced");
  const { settings } = useAppSettings();
  const queryClient = useQueryClient();
  const configDir = game?.config_dir ?? "";
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
  const defaultOverrideName = t("defaultPresetName");
  const overrideNameTouchedRef = useRef(false);
  const [overrideName, setOverrideNameState] = useState(defaultOverrideName);
  const [message, setMessage] = useState<string>();
  const [applyError, setApplyError] = useState<string>();
  const [activeCategory, setActiveCategory] = useState<string>(ALL_CATEGORY);
  const [search, setSearch] = useState("");
  const [debouncedSearch, setDebouncedSearch] = useState("");
  const deferredSearch = useDeferredValue(debouncedSearch);
  const [engineEnabled, setEngineEnabled] = useState<Set<string>>(new Set());

  const { panel, setPanel, filterMode, setFilterMode } = useEditorPanelState(
    game?.id,
    settings.defaultEditorPanel,
  );

  const setPanelWithCategoryReset = useCallback(
    (next: Parameters<typeof setPanel>[0]) => {
      setPanel(next);
      setActiveCategory(ALL_CATEGORY);
    },
    [setPanel],
  );

  useEffect(() => {
    const handle = window.setTimeout(() => setDebouncedSearch(search), 180);
    return () => window.clearTimeout(handle);
  }, [search]);

  useEffect(() => {
    setMessage(undefined);
    setApplyError(undefined);
    paramsDirtyRef.current = false;
    overrideNameTouchedRef.current = false;
    setOverrideNameState(defaultOverrideName);
  }, [game?.id, defaultOverrideName]);

  useEffect(() => {
    if (!overrideNameTouchedRef.current) {
      setOverrideNameState(defaultOverrideName);
    }
  }, [defaultOverrideName]);

  const setOverrideName = useCallback((value: string) => {
    overrideNameTouchedRef.current = true;
    setOverrideNameState(value);
  }, []);

  const { data: parameters = [], isLoading, isFetching } = useQuery({
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
        game?.id,
        game?.install_dir,
        game?.engine_family,
        game?.engine_version,
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
  const lastDiskRefreshRef = useRef(0);
  const FOCUS_DISK_REFRESH_MS = 60_000;

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

  const visibleParams = useMemo(
    () => params.filter((p) => isParamVisible(p, gpu)),
    [params, gpu],
  );

  useEffect(() => {
    if (paramsDirtyRef.current) return;
    setParams(normalizedParameters);
    setEngineEnabled(initialEngineEnabledKeys(normalizedParameters));
  }, [normalizedParameters]);

  const panelParams = useMemo(
    () => filterParamsByPanel(visibleParams, panel),
    [visibleParams, panel],
  );

  const modeFilteredParams = useMemo(
    () => filterParamsByMode(panelParams, filterMode, panel, deferredSearch),
    [panelParams, filterMode, panel, deferredSearch],
  );

  const categories = useMemo(
    () => buildCategoryList(modeFilteredParams),
    [modeFilteredParams],
  );

  useEffect(() => {
    if (categories.length && !categories.some((c) => c.cat === activeCategory)) {
      setActiveCategory(categories[0].cat);
    }
  }, [categories, activeCategory, panel]);

  const categoryFilteredParams = useMemo(
    () => filterParamsByCategory(modeFilteredParams, activeCategory),
    [modeFilteredParams, activeCategory],
  );

  const searchedParams = useMemo(
    () => filterParamsBySearch(categoryFilteredParams, deferredSearch),
    [categoryFilteredParams, deferredSearch],
  );

  const engineSortEnabled = shouldSortByEngineEnabled(activeCategory);
  const engineSortSet = engineSortEnabled ? engineEnabled : EMPTY_ENGINE_ENABLED;
  const filteredParams = useMemo(
    () => sortParamsForEngineCategory(searchedParams, activeCategory, engineSortSet),
    [searchedParams, activeCategory, engineSortSet],
  );

  const engineStats = useMemo(
    () => countEngineStats(panelParams, engineEnabled),
    [panelParams, engineEnabled],
  );

  const catalogStats = useMemo(() => {
    const known = panelParams.filter((p) => p.known).length;
    return {
      known,
      unknown: panelParams.length - known,
      total: panelParams.length,
    };
  }, [panelParams]);

  const editableCategories = useMemo(
    () => resolveEditableCategories(parameters, EDITABLE_FOR_APPLY),
    [parameters],
  );

  const pendingSummary = useMemo(() => {
    const { files, removals } = buildCustomChanges(
      filterParamsByPanel(params, panel),
      parameters,
      gpu,
      engineEnabled,
      editableCategories,
    );
    const summary = countPendingChanges(files, removals);
    const pendingKeySet = collectPendingKeys(files, removals);
    const conflictKeys = detectSgEngineConflicts(params, pendingKeySet, engineEnabled);
    return { ...summary, conflictKeys };
  }, [params, panel, parameters, gpu, engineEnabled, editableCategories]);

  const updateParam = useCallback((key: string, section: string, file: string, value: string) => {
    paramsDirtyRef.current = true;
    setParams((prev) =>
      applyParamDependencies(prev, { key, section, file, value }, gpu),
    );
  }, [gpu]);

  const toggleEngineParam = useCallback((p: GameParameter, enabled: boolean) => {
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
  }, [updateParam]);

  const discardChanges = () => {
    paramsDirtyRef.current = false;
    setParams(parameters);
    setEngineEnabled(initialEngineEnabledKeys(parameters));
    setApplyError(undefined);
    setMessage(undefined);
  };

  const {
    applyCustomMutation,
    saveOverrideMutation,
    applyOverrideMutation,
    deleteOverrideMutation,
    importOverrideMutation,
  } = useEditorMutations({
    game,
    configDir,
    runningExeName: runningExeName ?? null,
    params,
    parameters,
    panel,
    gpu,
    engineEnabled,
    editableCategories,
    overrideName,
    activeGameIdRef,
    setMessage,
    setApplyError,
    onApplied: () => {
      paramsDirtyRef.current = false;
    },
    t,
  });

  return {
    game,
    configDir,
    runningExeName: runningExeName ?? null,
    gameRunning,
    gpu,
    limits,
    overrides,
    message,
    applyError,
    setApplyError,
    activeCategory,
    setActiveCategory,
    search,
    setSearch,
    engineEnabled,
    panel,
    setPanel: setPanelWithCategoryReset,
    filterMode,
    setFilterMode,
    categories,
    filteredParams,
    engineStats,
    catalogStats,
    pendingChangesCount: pendingSummary.total,
    pendingChangesBreakdown: pendingSummary.breakdown,
    pendingConflictKeys: pendingSummary.conflictKeys,
    conflictCount: pendingSummary.conflictKeys.size,
    parametersLoading,
    overrideName,
    setOverrideName,
    updateParam,
    toggleEngineParam,
    discardChanges,
    applyCustomMutation,
    saveOverrideMutation,
    applyOverrideMutation,
    deleteOverrideMutation,
    importOverrideMutation,
    showEngineIniHint: panel === "advanced" && ENGINE_CATEGORIES.has(activeCategory),
  };
}

export type AdvancedEditorState = ReturnType<typeof useAdvancedEditorState>;
