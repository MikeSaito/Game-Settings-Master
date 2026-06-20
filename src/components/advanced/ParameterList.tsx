import { useVirtualizer } from "@tanstack/react-virtual";
import { Search } from "lucide-react";
import { useRef } from "react";
import { useTranslation } from "react-i18next";
import { ParameterCard } from "../ParameterCard";
import { EmptyState } from "../ui/EmptyState";
import { paramRowKey } from "../../lib/advancedEditorFilters";
import { filterSelectOptions } from "../../lib/gpuCompat";
import { getDependencyLabel } from "../../lib/paramDependencies";
import {
  isEngineEnabled,
  isEngineToggleable,
} from "../../lib/engineParams";
import type { GpuCapabilities, GameParameter } from "../../lib/types";

const ROW_ESTIMATE_PX = 120;

interface Props {
  filteredParams: GameParameter[];
  search: string;
  parametersLoading: boolean;
  gpu: GpuCapabilities | undefined;
  engineEnabled: Set<string>;
  onUpdateParam: (key: string, section: string, file: string, value: string) => void;
  onToggleEngineParam: (param: GameParameter, enabled: boolean) => void;
}

export function ParameterList({
  filteredParams,
  search,
  parametersLoading,
  gpu,
  engineEnabled,
  onUpdateParam,
  onToggleEngineParam,
}: Props) {
  const { t } = useTranslation("advanced");
  const parentRef = useRef<HTMLDivElement>(null);

  const virtualizer = useVirtualizer({
    count: filteredParams.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => ROW_ESTIMATE_PX,
    overscan: 6,
    measureElement: (el) => el.getBoundingClientRect().height,
  });

  if (parametersLoading) {
    return (
      <div className="flex flex-col items-center gap-3 py-16">
        <span className="h-8 w-8 animate-spin rounded-full border-2 border-[var(--color-border)] border-t-[var(--color-accent)]" />
        <p className="text-sm text-body">{t("loadingParams")}</p>
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
      className="max-h-[min(70vh,900px)] overflow-y-auto"
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
          const toggleable = isEngineToggleable(param);
          const enabled = isEngineEnabled(param, engineEnabled);
          return (
            <div
              key={paramRowKey(param)}
              ref={virtualizer.measureElement}
              data-index={virtualRow.index}
              className="absolute left-0 top-0 w-full pb-3"
              style={{ transform: `translateY(${virtualRow.start}px)` }}
            >
              <ParameterCard
                param={param}
                editable={param.editable && enabled}
                engineToggleable={toggleable}
                engineEnabled={enabled}
                selectOptions={filterSelectOptions(param, gpu) ?? undefined}
                dependencyLabel={getDependencyLabel(param.key) ?? undefined}
                onEngineToggle={(on) => onToggleEngineParam(param, on)}
                onChange={
                  param.editable && enabled
                    ? (value) =>
                        onUpdateParam(param.key, param.section, param.file, value)
                    : undefined
                }
              />
            </div>
          );
        })}
      </div>
    </div>
  );
}
