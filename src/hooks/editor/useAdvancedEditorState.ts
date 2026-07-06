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
  analyzeSgEngineConflictGroups,
  buildCustomChanges,
  buildIniSnapshot,
  EMPTY_INI_SNAPSHOT,
  collectPendingKeys,
  comboIssuesForKey,
  defaultValueFor,
  detectSgEngineConflicts,
  initialEngineEnabledKeys,
  resolveConflictKeepSg,
  validateApplyPlan,
  validateOverridePlan,
  mergePanelValidationIssues,
  validationIssueSignature,
  hasBlockingErrors,
} from "@/lib/editor";
import type { GameOverride } from "@/lib/core";
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
    limitsLoading,
    overrides,
    gpu,
    gpuLoading,
    gpuUnavailable,
    gameConfig,
    gameConfigLoading,
    gameConfigFetched,
    extraIniAvailable,
    paramsDirtyRef,
  } = useEditorQueries(game);

  const activeGameIdRef = useActiveGameIdRef(game?.id);
  const defaultOverrideName = t("defaultPresetName");
  const overrideNameTouchedRef = useRef(false);
  const [overrideName, setOverrideNameState] = useState(defaultOverrideName);
  const [message, setMessage] = useState<string>();
  const [applyError, setApplyError] = useState<string>();
  const [applyWarningsAcknowledged, setApplyWarningsAcknowledged] = useState(false);
  const [pendingPresetApply, setPendingPresetApply] = useState<GameOverride | null>(null);
  const [presetApplyWarningsAcknowledged, setPresetApplyWarningsAcknowledged] = useState(false);
  const [applyingPresetName, setApplyingPresetName] = useState<string | null>(null);
  const [activeCategory, setActiveCategory] = useState<string>(ALL_CATEGORY);
  const [search, setSearch] = useState("");
  const [debouncedSearch, setDebouncedSearch] = useState("");
  const deferredSearch = useDeferredValue(debouncedSearch);

  const { panel, setPanel, filterMode, setFilterMode } = useEditorPanelState(
    game?.id,
    settings.defaultEditorPanel,
  );

  const shippedIniSnapshotGameRef = useRef<string | undefined>(undefined);
  const shippedIniSnapshotCacheRef = useRef<ReadonlySet<string>>(EMPTY_INI_SNAPSHOT);

  const shippedIniKeys = useMemo(() => {
    if (!game?.id) {
      shippedIniSnapshotGameRef.current = undefined;
      shippedIniSnapshotCacheRef.current = EMPTY_INI_SNAPSHOT;
      return EMPTY_INI_SNAPSHOT;
    }
    if (parametersLoading || normalizedParameters.length === 0) {
      return shippedIniSnapshotCacheRef.current;
    }
    if (shippedIniSnapshotGameRef.current !== game.id) {
      shippedIniSnapshotGameRef.current = game.id;
      shippedIniSnapshotCacheRef.current = buildIniSnapshot(normalizedParameters);
    }
    return shippedIniSnapshotCacheRef.current;
  }, [game?.id, normalizedParameters, parametersLoading]);

  const { params, setParams, engineEnabled, setEngineEnabled } = useEditorParamDraft(
    normalizedParameters,
    paramsDirtyRef,
    shippedIniKeys,
  );

  const {
    categories,
    filteredParams,
    engineStats,
    gusIniStats,
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
    shippedIniKeys,
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
    setApplyWarningsAcknowledged(false);
    setPendingPresetApply(null);
    setPresetApplyWarningsAcknowledged(false);
    setApplyingPresetName(null);
    paramsDirtyRef.current = false;
    overrideNameTouchedRef.current = false;
  }, [game?.id, paramsDirtyRef]);

  useEffect(() => {
    if (!overrideNameTouchedRef.current) {
      setOverrideNameState(defaultOverrideName);
    }
  }, [defaultOverrideName]);

  useEffect(() => {
    if (!categories.length) return;
    if (categories.some((c) => c.cat === activeCategory)) return;
    const nextCategory = categories[0]?.cat;
    if (nextCategory && nextCategory !== activeCategory) {
      setActiveCategory(nextCategory);
    }
  }, [categories, activeCategory, panel]);

  useEffect(() => {
    if (panel !== "extra" || gameConfigLoading || !gameConfigFetched) return;
    if (!extraIniAvailable) {
      setPanelWithCategoryReset("basic");
    }
  }, [panel, extraIniAvailable, gameConfigLoading, gameConfigFetched, setPanelWithCategoryReset]);

  const validationPlanContext = useMemo(() => {
    if (!game) return null;
    return {
      game,
      params,
      parameters,
      gpu,
      engineEnabled,
      limits,
      limitsPending: limitsLoading,
      gpuPending: gpuLoading,
      gpuUnavailable,
      editableCategories,
      shippedIniKeys,
    };
  }, [
    game,
    params,
    parameters,
    gpu,
    engineEnabled,
    limits,
    limitsLoading,
    gpuLoading,
    gpuUnavailable,
    editableCategories,
    shippedIniKeys,
  ]);

  const validationIssues = useMemo(() => {
    if (!validationPlanContext) return [];
    if (panel === "extra" || panel === "backups" || panel === "presets") {
      return mergePanelValidationIssues(validationPlanContext);
    }
    return validateApplyPlan({ ...validationPlanContext, panel });
  }, [validationPlanContext, panel]);

  const validationIssuesSignature = useMemo(
    () => validationIssueSignature(validationIssues),
    [validationIssues],
  );
  const prevValidationSignatureRef = useRef("");

  useEffect(() => {
    if (prevValidationSignatureRef.current === validationIssuesSignature) return;
    prevValidationSignatureRef.current = validationIssuesSignature;
    setApplyWarningsAcknowledged(false);
  }, [validationIssuesSignature]);

  const presetValidationContext = useMemo(() => {
    if (!game) return null;
    return {
      game,
      params: parameters,
      gpu,
      limits,
      limitsPending: limitsLoading,
      gpuPending: gpuLoading,
      gpuUnavailable,
      engineEnabled: initialEngineEnabledKeys(parameters, shippedIniKeys),
      shippedIniKeys,
    };
  }, [
    game,
    parameters,
    gpu,
    limits,
    limitsLoading,
    gpuLoading,
    gpuUnavailable,
    shippedIniKeys,
  ]);

  const getPresetValidationIssues = useCallback(
    (override: GameOverride) => {
      if (!presetValidationContext) return [];
      return validateOverridePlan(
        override.files,
        override.removals,
        presetValidationContext,
      );
    },
    [presetValidationContext],
  );

  const presetApplyIssues = useMemo(
    () => (pendingPresetApply ? getPresetValidationIssues(pendingPresetApply) : []),
    [pendingPresetApply, getPresetValidationIssues],
  );

  const presetApplyIssuesSignature = useMemo(
    () => validationIssueSignature(presetApplyIssues),
    [presetApplyIssues],
  );
  const prevPresetValidationSignatureRef = useRef("");

  useEffect(() => {
    if (!pendingPresetApply) {
      prevPresetValidationSignatureRef.current = "";
      return;
    }
    if (prevPresetValidationSignatureRef.current === presetApplyIssuesSignature) return;
    prevPresetValidationSignatureRef.current = presetApplyIssuesSignature;
    setPresetApplyWarningsAcknowledged(false);
  }, [presetApplyIssuesSignature, pendingPresetApply]);

  const comboWarningsByKey = useMemo(() => {
    const comboOnly = validationIssues.filter((issue) => issue.code.startsWith("combo_"));
    const map = new Map<string, typeof comboOnly>();
    for (const param of params) {
      const related = comboIssuesForKey(comboOnly, param.key);
      if (related.length > 0) map.set(param.key.toLowerCase(), related);
    }
    return map;
  }, [validationIssues, params]);

  const setOverrideName = useCallback((value: string) => {
    overrideNameTouchedRef.current = true;
    setOverrideNameState(value);
  }, []);

  const pendingSummary = useMemo(() => {
    const { files, removals } = buildCustomChanges(
      params,
      parameters,
      gpu,
      engineEnabled,
      editableCategories,
      panel,
      shippedIniKeys,
    );
    const summary = countPendingChanges(files, removals);
    const pendingKeySet = collectPendingKeys(files, removals);
    const conflictKeys = detectSgEngineConflicts(
      params,
      pendingKeySet,
      engineEnabled,
      shippedIniKeys,
    );
    const conflictGroups = analyzeSgEngineConflictGroups(
      params,
      pendingKeySet,
      engineEnabled,
      shippedIniKeys,
    );
    return { ...summary, conflictKeys, conflictGroups };
  }, [params, panel, parameters, gpu, engineEnabled, editableCategories, shippedIniKeys]);

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
    setEngineEnabled(initialEngineEnabledKeys(parameters, shippedIniKeys));
    setApplyError(undefined);
    setMessage(undefined);
  };

  const resolveSgConflict = useCallback(
    (sgKey: string) => {
      const group = pendingSummary.conflictGroups.find((g) => g.sgKey === sgKey);
      if (!group) return;
      paramsDirtyRef.current = true;
      const { params: nextParams, engineEnabled: nextEnabled } = resolveConflictKeepSg(
        group,
        params,
        parameters,
        engineEnabled,
        shippedIniKeys,
      );
      setParams(nextParams);
      setEngineEnabled(nextEnabled);
      setMessage(t("conflict.resolved", { sg: group.sgParam.key }));
    },
    [
      pendingSummary.conflictGroups,
      params,
      parameters,
      engineEnabled,
      shippedIniKeys,
      paramsDirtyRef,
      setParams,
      setEngineEnabled,
      t,
    ],
  );

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
    limits,
    limitsPending: limitsLoading,
    gpuPending: gpuLoading,
    gpuUnavailable,
    engineEnabled,
    editableCategories,
    shippedIniKeys,
    overrideName,
    validationIssues,
    applyWarningsAcknowledged,
    activeGameIdRef,
    setMessage,
    setApplyError,
    onApplied: () => {
      paramsDirtyRef.current = false;
    },
    onPresetApplied: () => {
      setPendingPresetApply(null);
      setPresetApplyWarningsAcknowledged(false);
      setApplyingPresetName(null);
    },
    onPresetApplyStart: (name: string) => setApplyingPresetName(name),
    onPresetApplyEnd: () => setApplyingPresetName(null),
    t,
  });

  const requestApplyPreset = useCallback(
    (override: GameOverride) => {
      if (applyOverrideMutation.isPending) return;
      setApplyError(undefined);
      const issues = getPresetValidationIssues(override);
      const samePending =
        pendingPresetApply?.game_id === override.game_id &&
        pendingPresetApply?.name === override.name;

      if (hasBlockingErrors(issues) || issues.some((issue) => issue.severity === "warning")) {
        if (!samePending) setPresetApplyWarningsAcknowledged(false);
        setPendingPresetApply(override);
        return;
      }
      applyOverrideMutation.mutate({ override, warningsAcknowledged: false });
    },
    [getPresetValidationIssues, applyOverrideMutation, pendingPresetApply],
  );

  const confirmPendingPresetApply = useCallback(() => {
    if (!pendingPresetApply) return;
    applyOverrideMutation.mutate({
      override: pendingPresetApply,
      warningsAcknowledged: presetApplyWarningsAcknowledged,
    });
  }, [pendingPresetApply, presetApplyWarningsAcknowledged, applyOverrideMutation]);

  const cancelPendingPresetApply = useCallback(() => {
    setPendingPresetApply(null);
    setPresetApplyWarningsAcknowledged(false);
  }, []);

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
    gusIniStats,
    catalogStats,
    pendingChangesCount: pendingSummary.total,
    pendingChangesBreakdown: pendingSummary.breakdown,
    pendingConflictKeys: pendingSummary.conflictKeys,
    conflictCount: pendingSummary.conflictKeys.size,
    conflictGroups: pendingSummary.conflictGroups,
    validationIssues,
    applyWarningsAcknowledged,
    setApplyWarningsAcknowledged,
    comboWarningsByKey,
    gameConfig,
    gameConfigLoading,
    extraIniAvailable,
    resolveSgConflict,
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
    pendingPresetApply,
    presetApplyIssues,
    presetApplyWarningsAcknowledged,
    setPresetApplyWarningsAcknowledged,
    requestApplyPreset,
    confirmPendingPresetApply,
    cancelPendingPresetApply,
    applyingPresetName,
    shippedIniKeys,
    showEngineIniHint: panel === "advanced" && ENGINE_CATEGORIES.has(activeCategory),
  };
}

export type AdvancedEditorState = ReturnType<typeof useAdvancedEditorState>;
