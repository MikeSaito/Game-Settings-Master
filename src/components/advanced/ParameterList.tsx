import { useVirtualizer } from "@tanstack/react-virtual";
import { Search } from "lucide-react";
import { memo, useCallback, useEffect, useMemo, useRef, useState, type FocusEvent, type MouseEvent } from "react";
import { useTranslation } from "react-i18next";
import { ParameterDetailPane } from "./ParameterDetailPane";
import { ParameterRow } from "./ParameterRow";
import { EmptyState } from "@/components/ds/Feedback";
import {
  EMPTY_INI_SNAPSHOT,
  getDependencyLabel,
  getParamSelectOptions,
  isEngineEnabled,
  isIniMembershipToggleable,
  paramRowKey,
} from "@/lib/editor";
import { formatValidationIssue } from "@/lib/editor/validation";
import type { ValidationIssue } from "@/lib/editor/validation";
import { cn } from "@/lib/core";
import type { GpuCapabilities, GameParameter } from "@/lib/core";

const ROW_ESTIMATE_PX = 96;

interface DetailContext {
  param: GameParameter;
  engineToggleable: boolean;
  engineEnabled: boolean;
}

interface Props {
  filteredParams: GameParameter[];
  search: string;
  parametersLoading: boolean;
  gpu: GpuCapabilities | undefined;
  engineEnabled: Set<string>;
  showEngineToggle?: boolean;
  /** When set, only GUS catalog extras use ini toggle (basic panel). */
  gusIniToggleOnly?: boolean;
  shippedIniKeys?: ReadonlySet<string>;
  pendingConflictKeys?: Set<string>;
  comboWarningsByKey?: Map<string, ValidationIssue[]>;
  onUpdateParam: (key: string, section: string, file: string, value: string) => void;
  onToggleEngineParam: (param: GameParameter, enabled: boolean) => void;
  className?: string;
}

interface ParameterListRowProps {
  param: GameParameter;
  gpu: GpuCapabilities | undefined;
  enabled: boolean;
  toggleable: boolean;
  hasConflict: boolean;
  conflictText: string;
  warningLabel?: string;
  shippedIniKeys: ReadonlySet<string>;
  onUpdateParam: (key: string, section: string, file: string, value: string) => void;
  onToggleEngineParam: (param: GameParameter, enabled: boolean) => void;
}

const ParameterListRow = memo(function ParameterListRow({
  param,
  gpu,
  enabled,
  toggleable,
  hasConflict,
  conflictText,
  warningLabel,
  shippedIniKeys,
  onUpdateParam,
  onToggleEngineParam,
}: ParameterListRowProps) {
  const selectOptions = useMemo(
    () => getParamSelectOptions(param, gpu),
    [param, gpu],
  );
  const dependencyLabel = useMemo(
    () => getDependencyLabel(param.key) ?? undefined,
    [param.key],
  );
  const conflictLabel = hasConflict ? conflictText : undefined;
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
      warningLabel={warningLabel}
      shippedIniKeys={shippedIniKeys}
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
  gusIniToggleOnly = false,
  shippedIniKeys = EMPTY_INI_SNAPSHOT,
  pendingConflictKeys,
  comboWarningsByKey,
  onUpdateParam,
  onToggleEngineParam,
  className,
}: Props) {
  const { t } = useTranslation("advanced");
  const parentRef = useRef<HTMLDivElement>(null);
  const listRootRef = useRef<HTMLDivElement>(null);
  const closeTimerRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const detailKeyRef = useRef<string | null>(null);
  const [detail, setDetail] = useState<DetailContext | null>(null);
  const conflictText = t("conflict.sgEngine");

  const showDetailFor = useCallback((ctx: DetailContext) => {
    const key = paramRowKey(ctx.param);
    if (closeTimerRef.current) clearTimeout(closeTimerRef.current);
    if (detailKeyRef.current === key) return;
    detailKeyRef.current = key;
    setDetail(ctx);
  }, []);

  const scheduleCloseDetail = useCallback(() => {
    closeTimerRef.current = setTimeout(() => {
      const root = listRootRef.current;
      const active = document.activeElement;
      if (root && active instanceof Node && root.contains(active)) {
        return;
      }
      detailKeyRef.current = null;
      setDetail(null);
    }, 150);
  }, []);

  const cancelCloseDetail = useCallback(() => {
    if (closeTimerRef.current) clearTimeout(closeTimerRef.current);
  }, []);

  const virtualizer = useVirtualizer({
    count: filteredParams.length,
    getScrollElement: () => parentRef.current,
    getItemKey: (index) => paramRowKey(filteredParams[index]),
    estimateSize: () => ROW_ESTIMATE_PX,
    overscan: 6,
    measureElement: (el) => Math.ceil(el.getBoundingClientRect().height),
  });

  const rowKeys = useMemo(
    () => filteredParams.map((param) => paramRowKey(param)).join("\0"),
    [filteredParams],
  );

  useEffect(() => {
    const el = parentRef.current;
    if (el && typeof el.scrollTo === "function") {
      el.scrollTo({ top: 0 });
    }
    if (typeof virtualizer.scrollToIndex === "function") {
      virtualizer.scrollToIndex(0);
    }
    setDetail(null);
    detailKeyRef.current = null;
    // Reset scroll only when search or visible row set changes — not on value edits.
    // eslint-disable-next-line react-hooks/exhaustive-deps -- virtualizer instance is stable enough here
  }, [search, rowKeys]);

  const revealDetailFromRow = useCallback(
    (row: HTMLElement) => {
      const indexRaw = row.dataset.index;
      const key = row.dataset.paramKey;
      if (indexRaw == null || key == null) return;
      const index = Number(indexRaw);
      if (!Number.isFinite(index)) return;
      const param = filteredParams[index];
      if (!param || paramRowKey(param) !== key) return;

      const toggleable =
        showEngineToggle &&
        (gusIniToggleOnly
          ? isIniMembershipToggleable(param, shippedIniKeys) && param.file === "GameUserSettings.ini"
          : isIniMembershipToggleable(param, shippedIniKeys));
      const enabled = toggleable
        ? isEngineEnabled(param, engineEnabled, shippedIniKeys)
        : true;

      showDetailFor({
        param,
        engineToggleable: toggleable,
        engineEnabled: enabled,
      });
    },
    [
      engineEnabled,
      filteredParams,
      gusIniToggleOnly,
      shippedIniKeys,
      showDetailFor,
      showEngineToggle,
    ],
  );

  const handleScrollAreaMouseOver = useCallback(
    (event: MouseEvent<HTMLDivElement>) => {
      const row = (event.target as HTMLElement).closest<HTMLElement>("[data-param-key]");
      if (!row) return;
      revealDetailFromRow(row);
    },
    [revealDetailFromRow],
  );

  const handleScrollAreaFocus = useCallback(
    (event: FocusEvent<HTMLDivElement>) => {
      const row = (event.target as HTMLElement).closest<HTMLElement>("[data-param-key]");
      if (!row) return;
      revealDetailFromRow(row);
    },
    [revealDetailFromRow],
  );

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
      ref={listRootRef}
      className={cn(
        "flex min-h-[320px] flex-1 flex-row overflow-hidden rounded-[var(--radius-panel)] border border-[var(--color-border)] bg-[var(--color-bg-soft)]",
        className,
      )}
      data-testid="parameter-list"
      onMouseLeave={scheduleCloseDetail}
    >
      <div
        ref={parentRef}
        className="min-h-0 min-w-0 flex-1 overflow-y-auto rounded-l-[var(--radius-panel)] bg-[var(--color-bg-soft)]"
        data-testid="parameter-list-scroll"
        onMouseOver={handleScrollAreaMouseOver}
        onFocusCapture={handleScrollAreaFocus}
      >
        <div
          className="relative w-full"
          style={{ height: `${virtualizer.getTotalSize()}px` }}
          data-testid="parameter-list-virtual"
          data-virtual-count={filteredParams.length}
        >
          {virtualizer.getVirtualItems().map((virtualRow) => {
            const param = filteredParams[virtualRow.index];
            const toggleable =
              showEngineToggle &&
              (gusIniToggleOnly
                ? isIniMembershipToggleable(param, shippedIniKeys) &&
                  param.file === "GameUserSettings.ini"
                : isIniMembershipToggleable(param, shippedIniKeys));
            const enabled = toggleable
              ? isEngineEnabled(param, engineEnabled, shippedIniKeys)
              : true;
            const hasConflict = pendingConflictKeys?.has(param.key.toLowerCase()) ?? false;
            const comboIssues = comboWarningsByKey?.get(param.key.toLowerCase()) ?? [];
            const warningLabel =
              !hasConflict && comboIssues.length > 0
                ? formatValidationIssue(t, comboIssues[0])
                : undefined;
            const rowKey = paramRowKey(param);
            return (
              <div
                key={rowKey}
                ref={virtualizer.measureElement}
                data-index={virtualRow.index}
                data-param-key={rowKey}
                className="absolute left-0 top-0 w-full"
                style={{ transform: `translateY(${virtualRow.start}px)` }}
              >
                <ParameterListRow
                  param={param}
                  gpu={gpu}
                  enabled={enabled}
                  toggleable={toggleable}
                  hasConflict={hasConflict}
                  conflictText={conflictText}
                  warningLabel={warningLabel}
                  shippedIniKeys={shippedIniKeys}
                  onUpdateParam={onUpdateParam}
                  onToggleEngineParam={onToggleEngineParam}
                />
              </div>
            );
          })}
        </div>
      </div>

      <ParameterDetailPane
        param={detail?.param ?? null}
        engineToggleable={detail?.engineToggleable}
        engineEnabled={detail?.engineEnabled}
        onPointerEnter={cancelCloseDetail}
        onPointerLeave={scheduleCloseDetail}
      />
    </div>
  );
}
