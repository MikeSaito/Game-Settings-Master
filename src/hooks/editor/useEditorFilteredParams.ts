import { useEffect, useMemo, useState, type MutableRefObject } from "react";
import {
  ALL_CATEGORY,
  buildCategoryList,
  countEngineStats,
  engineParamId,
  filterParamsByCategory,
  filterParamsBySearch,
  initialEngineEnabledKeys,
  resolveEditableCategories,
  shouldSortByEngineEnabled,
  sortParamsForEngineCategory,
} from "@/lib/editor";
import { filterParamsByMode, filterParamsByPanel, type EditorPanel } from "@/lib/routing";
import { isParamVisible } from "@/lib/gpu";
import type { GameParameter, GpuCapabilities } from "@/lib/core";
import type { EditorFilterMode } from "@/lib/routing";

const EMPTY_ENGINE_ENABLED = new Set<string>();

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

interface FilterOptions {
  panel: EditorPanel;
  filterMode: EditorFilterMode;
  deferredSearch: string;
  gpu: GpuCapabilities | undefined;
}

export function useEditorParamDraft(
  normalizedParameters: GameParameter[],
  paramsDirtyRef: MutableRefObject<boolean>,
) {
  const [params, setParams] = useState<GameParameter[]>([]);
  const [engineEnabled, setEngineEnabled] = useState<Set<string>>(new Set());

  useEffect(() => {
    if (paramsDirtyRef.current) return;

    setParams((current) =>
      current === normalizedParameters ? current : normalizedParameters,
    );

    setEngineEnabled((current) => {
      const next = initialEngineEnabledKeys(normalizedParameters);
      if (
        current.size === next.size &&
        [...current].every((key) => next.has(key))
      ) {
        return current;
      }
      return next;
    });
  }, [normalizedParameters, paramsDirtyRef]);

  return { params, setParams, engineEnabled, setEngineEnabled };
}

export function useEditorFilteredParams({
  params,
  panel,
  filterMode,
  deferredSearch,
  gpu,
  engineEnabled,
  activeCategory,
  parameters,
}: FilterOptions & {
  params: GameParameter[];
  engineEnabled: Set<string>;
  activeCategory: string;
  parameters: GameParameter[];
}) {
  const visibleParams = useMemo(
    () => params.filter((p) => isParamVisible(p, gpu)),
    [params, gpu],
  );

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

  return {
    panelParams,
    categories,
    filteredParams,
    engineStats,
    catalogStats,
    editableCategories,
    engineParamId,
  };
}

export { ALL_CATEGORY, EDITABLE_FOR_APPLY };
