import { useCallback, useEffect, useMemo, useRef, useState, useDeferredValue } from "react";
import { useTranslation } from "react-i18next";
import { useWorkspacePreset } from "@/context/GameWorkspaceContext";
import { useAppSettings } from "@/hooks/app/useAppSettings";
import { useActiveGameIdRef } from "@/hooks/game/useActiveGameIdRef";
import { countPendingChanges } from "@/hooks/editor/editorStateUtils";
import {
  ALL_CATEGORY,
  useEditorFilteredParams,
  useEditorParamDraft,
} from "@/hooks/editor/useEditorFilteredParams";
import { useEditorMutations } from "@/hooks/editor/useEditorMutations";
import { useEditorPanelState } from "@/hooks/editor/useEditorPanelState";
import { useEditorQueries } from "@/hooks/editor/useEditorQueries";
import {
  applyParamDependencies,
  buildCustomChanges,
  collectPendingKeys,
  defaultValueFor,
  detectSgEngineConflicts,
  initialEngineEnabledKeys,
} from "@/lib/editor";
import { filterParamsByPanel } from "@/lib/routing";
import { ENGINE_CATEGORIES } from "@/lib/editor";
import type { GameParameter, GameProfile } from "@/lib/core";

export function useAdvancedEditorState(game: GameProfile | null) {
  const { t } = useTranslation("advanced");
  const { settings } = useAppSettings();
  const configDir = game?.config_dir ?? "";

  useWorkspacePreset(t("title"), "selected", !!configDir);

  const {
    runningExeName,
    gameRunning,
    parameters,
    parametersLoading,
    normalizedParameters,
    limits,
    overrides,
    gpu,
    paramsDirtyRef,
  } = useEditorQueries(game);

  const activeGameIdRef = useActiveGameIdRef(game?.id);
  const defaultOverrideName = t("defaultPresetName");
  const overrideNameTouchedRef = useRef(false);
  const [overrideName, setOverrideNameState] = useState(defaultOverrideName);
  const [message, setMessage] = useState<string>();
  const [applyError, setApplyError] = useState<string>();
  const [activeCategory, setActiveCategory] = useState<string>(ALL_CATEGORY);
  const [search, setSearch] = useState("");
  const [debouncedSearch, setDebouncedSearch] = useState("");
  const deferredSearch = useDeferredValue(debouncedSearch);

  const { panel, setPanel, filterMode, setFilterMode } = useEditorPanelState(
    game?.id,
    settings.defaultEditorPanel,
  );

  const { params, setParams, engineEnabled, setEngineEnabled } = useEditorParamDraft(
    normalizedParameters,
    paramsDirtyRef,
  );

  const {
    categories,
    filteredParams,
    engineStats,
    catalogStats,
    editableCategories,
    engineParamId,
  } = useEditorFilteredParams({
    params,
    panel,
    filterMode,
    deferredSearch,
    gpu,
    engineEnabled,
    activeCategory,
    parameters,
  });

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
  }, [game?.id, defaultOverrideName, paramsDirtyRef]);

  useEffect(() => {
    if (!overrideNameTouchedRef.current) {
      setOverrideNameState(defaultOverrideName);
    }
  }, [defaultOverrideName]);

  useEffect(() => {
    if (categories.length && !categories.some((c) => c.cat === activeCategory)) {
      setActiveCategory(categories[0].cat);
    }
  }, [categories, activeCategory, panel]);

  const setOverrideName = useCallback((value: string) => {
    overrideNameTouchedRef.current = true;
    setOverrideNameState(value);
  }, []);

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

  const updateParam = useCallback(
    (key: string, section: string, file: string, value: string) => {
      paramsDirtyRef.current = true;
      setParams((prev) =>
        applyParamDependencies(prev, { key, section, file, value }, gpu),
      );
    },
    [gpu, paramsDirtyRef, setParams],
  );

  const toggleEngineParam = useCallback(
    (p: GameParameter, enabled: boolean) => {
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
    },
    [engineParamId, paramsDirtyRef, setEngineEnabled, updateParam],
  );

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
