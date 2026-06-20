import { useVirtualizer } from "@tanstack/react-virtual";
import { Search } from "lucide-react";
import { memo, useCallback, useEffect, useMemo, useRef } from "react";
import { useTranslation } from "react-i18next";
import { ParameterRow } from "./ParameterRow";
import { EmptyState } from "../ds/Feedback";
import { paramRowKey } from "../../lib/advancedEditorFilters";
import { getParamSelectOptions } from "../../lib/paramSelectOptions";
import { getDependencyLabel } from "../../lib/paramDependencies";
import {
  isEngineEnabled,
  isEngineToggleable,
} from "../../lib/engineParams";
import type { GpuCapabilities, GameParameter } from "../../lib/types";

const ROW_ESTIMATE_PX = 74;

interface Props {
  filteredParams: GameParameter[];
  search: string;
  parametersLoading: boolean;
  gpu: GpuCapabilities | undefined;
  engineEnabled: Set<string>;
  showEngineToggle?: boolean;
  pendingConflictKeys?: Set<string>;
  onUpdateParam: (key: string, section: string, file: string, value: string) => void;
  onToggleEngineParam: (param: GameParameter, enabled: boolean) => void;
}

interface ParameterListRowProps {
  param: GameParameter;
  gpu: GpuCapabilities | undefined;
  engineEnabled: Set<string>;
  showEngineToggle: boolean;
  pendingConflictKeys?: Set<string>;
  conflictText: string;
  onUpdateParam: (key: string, section: string, file: string, value: string) => void;
  onToggleEngineParam: (param: GameParameter, enabled: boolean) => void;
}

const ParameterListRow = memo(function ParameterListRow({
  param,
  gpu,
  engineEnabled,
  showEngineToggle,
  pendingConflictKeys,
  conflictText,
  onUpdateParam,
  onToggleEngineParam,
}: ParameterListRowProps) {
  const toggleable = showEngineToggle && isEngineToggleable(param);
  const enabled = toggleable ? isEngineEnabled(param, engineEnabled) : true;
  const selectOptions = useMemo(
    () => getParamSelectOptions(param, gpu),
    [param, gpu],
  );
  const dependencyLabel = useMemo(
    () => getDependencyLabel(param.key) ?? undefined,
    [param.key],
  );
  const conflictLabel = pendingConflictKeys?.has(param.key.toLowerCase())
    ? conflictText
    : undefined;
  const handleEngineToggle = useCallback(
    (on: boolean) => onToggleEngineParam(param, on),
    [onToggleEngineParam, param],
  );
  const handleChange = useCallback(
    (value: string) => onUpdateParam(param.key, param.section, param.file, value),
    [onUpdateParam, param.file, param.key, param.section],
  );

  return (
    <ParameterRow
      param={param}
      editable={param.editable && enabled}
      engineToggleable={toggleable}
      engineEnabled={enabled}
      selectOptions={selectOptions}
      dependencyLabel={dependencyLabel}
      conflictLabel={conflictLabel}
      onEngineToggle={handleEngineToggle}
      onChange={param.editable && enabled ? handleChange : undefined}
    />
  );
});

export function ParameterList({
  filteredParams,
  search,
  parametersLoading,
  gpu,
  engineEnabled,
  showEngineToggle = true,
  pendingConflictKeys,
  onUpdateParam,
  onToggleEngineParam,
}: Props) {
  const { t } = useTranslation("advanced");
  const parentRef = useRef<HTMLDivElement>(null);
  const conflictText = t("conflict.sgEngine");

  const virtualizer = useVirtualizer({
    count: filteredParams.length,
    getScrollElement: () => parentRef.current,
    getItemKey: (index) => paramRowKey(filteredParams[index]),
    estimateSize: () => ROW_ESTIMATE_PX,
    overscan: 6,
    measureElement: (el) => el.getBoundingClientRect().height,
  });

  useEffect(() => {
    if (typeof parentRef.current?.scrollTo === "function") {
      parentRef.current.scrollTo({ top: 0 });
    }
    if (typeof virtualizer.scrollToIndex === "function") {
      virtualizer.scrollToIndex(0);
    }
  }, [filteredParams, search, virtualizer]);

  if (parametersLoading) {
    return (
      <div className="flex flex-col items-center gap-3 py-16">
        <span className="h-8 w-8 animate-spin rounded-full border-2 border-[var(--color-border)] border-t-[var(--color-accent)]" />
        <p className="text-sm text-[var(--color-text-secondary)]">{t("loadingParams")}</p>
      </div>
    );
  }

  if (filteredParams.length === 0) {
    return (
      <EmptyState
        icon={Search}
        title={search ? t("emptyFiltered.titleSearch") : t("emptyFiltered.titleEmpty")}
        description={
          search ? t("emptyFiltered.descSearch") : t("emptyFiltered.descEmpty")
        }
        className="py-12"
      />
    );
  }

  return (
    <div
      ref={parentRef}
      className="max-h-[min(720px,calc(100dvh-16rem))] min-h-[320px] overflow-y-auto rounded-[var(--radius-panel)] border border-[var(--color-border)] bg-[var(--color-bg-soft)]"
      data-testid="parameter-list-scroll"
    >
      <div
        className="relative w-full"
        style={{ height: `${virtualizer.getTotalSize()}px` }}
        data-testid="parameter-list-virtual"
        data-virtual-count={filteredParams.length}
      >
        {virtualizer.getVirtualItems().map((virtualRow) => {
          const param = filteredParams[virtualRow.index];
          return (
            <div
              key={paramRowKey(param)}
              ref={virtualizer.measureElement}
              data-index={virtualRow.index}
              className="absolute left-0 top-0 w-full"
              style={{ transform: `translateY(${virtualRow.start}px)` }}
            >
              <ParameterListRow
                param={param}
                gpu={gpu}
                engineEnabled={engineEnabled}
                showEngineToggle={showEngineToggle}
                pendingConflictKeys={pendingConflictKeys}
                conflictText={conflictText}
                onUpdateParam={onUpdateParam}
                onToggleEngineParam={onToggleEngineParam}
              />
            </div>
          );
        })}
      </div>
    </div>
  );
}
