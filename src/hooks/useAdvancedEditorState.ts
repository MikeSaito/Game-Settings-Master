import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { currentLanguage } from "../i18n";
import { useAppWindowFocused } from "../context/AppWindowFocusProvider";
import { useWorkspacePreset } from "../context/GameWorkspaceContext";
import { useBackgroundSafeEnabled } from "./useBackgroundSafeEnabled";
import { useAppSettings } from "./useAppSettings";
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
  ALL_CATEGORY,
  countEngineStats,
  filterParamsByCategoryAndSearch,
  normalizeParameterCategories,
} from "../lib/advancedEditorFilters";
import {
  type EditorPanel,
  type EditorFilterMode,
  defaultFilterMode,
  filterParamsByMode,
  filterParamsByPanel,
  panelFromHash,
  readStoredFilterMode,
  readStoredPanel,
  syncPanelToHash,
  writeStoredFilterMode,
  writeStoredPanel,
} from "../lib/editorPanels";
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

function countPendingChanges(
  files: Record<string, Record<string, Record<string, string>>>,
  removals: Record<string, Record<string, string[]>>,
  allParams: GameParameter[],
) {
  const breakdown = { sg: 0, display: 0, engine: 0 };
  const pendingKeys = new Set<string>();
  let total = 0;

  for (const [file, sections] of Object.entries(files)) {
    for (const entries of Object.values(sections)) {
      for (const key of Object.keys(entries)) {
        pendingKeys.add(key);
        total += 1;
        if (key.startsWith("sg.")) breakdown.sg += 1;
        else if (file === "GameUserSettings.ini") {
          breakdown.display += 1;
        } else {
          breakdown.engine += 1;
        }
      }
    }
  }

  for (const [file, sections] of Object.entries(removals)) {
    for (const keys of Object.values(sections)) {
      for (const key of keys) {
        pendingKeys.add(key);
        total += 1;
        if (key.startsWith("sg.")) breakdown.sg += 1;
        else if (file === "GameUserSettings.ini") breakdown.display += 1;
        else breakdown.engine += 1;
      }
    }
  }

  const lowerKeys = [...pendingKeys].map((key) => key.toLowerCase());
  const hasSgShadow = lowerKeys.includes("sg.shadowquality");
  const shadowRKeys = lowerKeys.filter((key) => key.startsWith("r.shadow"));
  const conflictKeys = new Set<string>();
  if (hasSgShadow && shadowRKeys.length > 0) {
    conflictKeys.add("sg.shadowquality");
    for (const key of shadowRKeys) conflictKeys.add(key);
  }
  if (!hasSgShadow && shadowRKeys.length > 0) {
    const activeSgShadow = allParams.some(
      (param) =>
        param.key.toLowerCase() === "sg.shadowquality" &&
        param.file === "GameUserSettings.ini" &&
        param.present_in_ini &&
        param.value.trim() !== "",
    );
    if (activeSgShadow) {
      for (const key of shadowRKeys) conflictKeys.add(key);
    }
  }

  return { total, breakdown, conflictKeys };
}

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
  const [engineEnabled, setEngineEnabled] = useState<Set<string>>(new Set());
  const [panel, setPanelState] = useState<EditorPanel>("basic");
  const [filterMode, setFilterModeState] = useState<EditorFilterMode>("recommended");

  useEffect(() => {
    const handle = window.setTimeout(() => setDebouncedSearch(search), 180);
    return () => window.clearTimeout(handle);
  }, [search]);

  useEffect(() => {
    if (!game?.id) return;
    const fromHash = panelFromHash();
    const stored = readStoredPanel(game.id);
    const nextPanel = fromHash ?? stored ?? settings.defaultEditorPanel;
    setPanelState(nextPanel);
    const storedFilter = readStoredFilterMode(game.id, nextPanel);
    setFilterModeState(storedFilter ?? defaultFilterMode(nextPanel));
  }, [game?.id, settings.defaultEditorPanel]);

  useEffect(() => {
    const onHashChange = () => {
      const fromHash = panelFromHash();
      if (!fromHash) return;
      setPanelState(fromHash);
      if (game?.id) {
        writeStoredPanel(game.id, fromHash);
        const storedFilter = readStoredFilterMode(game.id, fromHash);
        setFilterModeState(storedFilter ?? defaultFilterMode(fromHash));
      }
    };
    window.addEventListener("hashchange", onHashChange);
    return () => window.removeEventListener("hashchange", onHashChange);
  }, [game?.id]);

  const setPanel = useCallback(
    (next: EditorPanel) => {
      setPanelState(next);
      if (game?.id) {
        writeStoredPanel(game.id, next);
        syncPanelToHash(next);
        const storedFilter = readStoredFilterMode(game.id, next);
        setFilterModeState(storedFilter ?? defaultFilterMode(next));
      } else {
        setFilterModeState(defaultFilterMode(next));
      }
      setActiveCategory(ALL_CATEGORY);
    },
    [game?.id],
  );

  const setFilterMode = useCallback(
    (mode: EditorFilterMode) => {
      setFilterModeState(mode);
      if (game?.id) writeStoredFilterMode(game.id, panel, mode);
    },
    [game?.id, panel],
  );

  useEffect(() => {
    setMessage(undefined);
    setApplyError(undefined);
    paramsDirtyRef.current = false;
    overrideNameTouchedRef.current = false;
    setOverrideNameState(defaultOverrideName);
  }, [game?.id]);

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
    setParams(normalizedParameters);
    setEngineEnabled(initialEngineEnabledKeys(normalizedParameters));
  }, [normalizedParameters]);

  const panelParams = useMemo(
    () => filterParamsByPanel(visibleParams, panel),
    [visibleParams, panel],
  );

  const modeFilteredParams = useMemo(
    () => filterParamsByMode(panelParams, filterMode, panel, debouncedSearch),
    [panelParams, filterMode, panel, debouncedSearch],
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

  const filteredParams = useMemo(
    () =>
      filterParamsByCategoryAndSearch(
        modeFilteredParams,
        activeCategory,
        debouncedSearch,
        engineEnabled,
      ),
    [modeFilteredParams, activeCategory, debouncedSearch, engineEnabled],
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
    return countPendingChanges(files, removals, params);
  }, [params, panel, parameters, gpu, engineEnabled, editableCategories]);

  const pendingChangesCount = pendingSummary.total;

  const buildChanges = () =>
    buildCustomChanges(
      filterParamsByPanel(params, panel),
      parameters,
      gpu,
      engineEnabled,
      editableCategories,
    );

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

  const applyCustomMutation = useMutation({
    mutationFn: async () => {
      const snapshot = { gameId: game!.id, configDir };
      const { files, removals } = buildChanges();
      if (
        Object.keys(files).length === 0 &&
        Object.keys(removals).length === 0
      ) {
        throw new Error(t("errors.noChanges"));
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
    panel,
    setPanel,
    filterMode,
    setFilterMode,
    categories,
    filteredParams,
    engineStats,
    catalogStats,
    pendingChangesCount,
    pendingChangesBreakdown: pendingSummary.breakdown,
    pendingConflictKeys: pendingSummary.conflictKeys,
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
    showEngineIniHint: panel === "advanced" && ENGINE_CATEGORIES.has(activeCategory),
  };
}

export type AdvancedEditorState = ReturnType<typeof useAdvancedEditorState>;
