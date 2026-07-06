import { useVirtualizer } from "@tanstack/react-virtual";
import { Search } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { EmptyState } from "@/components/ds/Feedback";
import { Input } from "@/components/ds/Field";
import type { GameConfig } from "@/lib/core";
import { EXTRA_INI_FILES } from "@/lib/ini/configFiles";

interface Props {
  gameConfig: GameConfig | undefined;
  loading?: boolean;
}

type ExtraIniRow = { file: string; section: string; key: string; value: string };

const ROW_ESTIMATE_PX = 32;
const GRID_COLS = "minmax(7rem,1fr) minmax(7rem,1.2fr) minmax(10rem,1.5fr) minmax(12rem,2fr)";

function flattenIni(
  file: string,
  sections: Record<string, Record<string, string>>,
): ExtraIniRow[] {
  const rows: ExtraIniRow[] = [];
  for (const [section, entries] of Object.entries(sections)) {
    for (const [key, value] of Object.entries(entries)) {
      rows.push({ file, section, key, value });
    }
  }
  return rows;
}

export function ExtraIniPanel({ gameConfig, loading }: Props) {
  const { t } = useTranslation("advanced");
  const [search, setSearch] = useState("");
  const parentRef = useRef<HTMLDivElement>(null);
  const q = search.trim().toLowerCase();

  const rows = useMemo(() => {
    if (!gameConfig?.files) return [];
    const all: ExtraIniRow[] = [];
    for (const file of EXTRA_INI_FILES) {
      const data = gameConfig.files[file];
      if (!data?.sections) continue;
      all.push(...flattenIni(file, data.sections));
    }
    if (!q) return all;
    return all.filter(
      (row) =>
        row.key.toLowerCase().includes(q) ||
        row.value.toLowerCase().includes(q) ||
        row.section.toLowerCase().includes(q) ||
        row.file.toLowerCase().includes(q),
    );
  }, [gameConfig, q]);

  const availableFiles = useMemo(() => {
    if (!gameConfig?.files) return [];
    return EXTRA_INI_FILES.filter((file) => file in gameConfig.files);
  }, [gameConfig]);

  const virtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => parentRef.current,
    getItemKey: (index) => `${rows[index].file}|${rows[index].section}|${rows[index].key}`,
    estimateSize: () => ROW_ESTIMATE_PX,
    overscan: 8,
  });

  const rowKeys = useMemo(
    () => rows.map((row) => `${row.file}|${row.section}|${row.key}`).join("\0"),
    [rows],
  );

  useEffect(() => {
    const el = parentRef.current;
    if (el && typeof el.scrollTo === "function") {
      el.scrollTo({ top: 0 });
    }
    if (typeof virtualizer.scrollToIndex === "function") {
      virtualizer.scrollToIndex(0);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps -- reset scroll on filter change only
  }, [search, rowKeys]);

  if (loading) {
    return (
      <div className="flex flex-col items-center gap-3 py-16">
        <span className="h-8 w-8 animate-spin rounded-full border-2 border-[var(--color-border)] border-t-[var(--color-accent)]" />
        <p className="text-sm text-[var(--color-text-secondary)]">{t("extra.loading")}</p>
      </div>
    );
  }

  if (availableFiles.length === 0) {
    return (
      <EmptyState
        icon={Search}
        title={t("extra.emptyTitle")}
        description={t("extra.emptyDesc")}
        className="py-12"
      />
    );
  }

  return (
    <div className="space-y-3">
      <div>
        <h2 className="text-lg font-semibold text-[var(--color-text)]">{t("mode.extraTitle")}</h2>
        <p className="mt-1 text-xs text-[var(--color-text-muted)]">{t("mode.extraBody")}</p>
      </div>

      <Input
        aria-label={t("extra.search")}
        placeholder={t("extra.searchPlaceholder")}
        value={search}
        onChange={(event) => setSearch(event.target.value)}
      />

      <div className="overflow-x-auto rounded-[var(--radius-panel)] border border-[var(--color-border)] bg-[var(--color-bg-soft)]">
        <div
          className="grid min-w-[520px] border-b border-[var(--color-border)] bg-[var(--color-surface)] text-xs text-[var(--color-text-muted)]"
          style={{ gridTemplateColumns: GRID_COLS }}
        >
          <div className="px-3 py-2 font-medium">{t("extra.colFile")}</div>
          <div className="px-3 py-2 font-medium">{t("extra.colSection")}</div>
          <div className="px-3 py-2 font-medium">{t("table.param")}</div>
          <div className="px-3 py-2 font-medium">{t("inIni")}</div>
        </div>

        {rows.length === 0 ? (
          <div className="px-3 py-8 text-center text-xs text-[var(--color-text-muted)]">
            {q ? t("extra.noSearch") : t("extra.noRows")}
          </div>
        ) : (
          <div
            ref={parentRef}
            className="max-h-[min(720px,calc(100dvh-16rem))] min-h-[240px] overflow-y-auto"
            data-testid="extra-ini-scroll"
          >
            <div
              className="relative min-w-[520px]"
              style={{ height: `${virtualizer.getTotalSize()}px` }}
              data-testid="extra-ini-virtual"
              data-virtual-count={rows.length}
            >
              {virtualizer.getVirtualItems().map((virtualRow) => {
                const row = rows[virtualRow.index];
                return (
                  <div
                    key={`${row.file}|${row.section}|${row.key}`}
                    data-index={virtualRow.index}
                    className="absolute left-0 top-0 grid w-full border-b border-[var(--color-border)] text-xs last:border-b-0"
                    style={{
                      gridTemplateColumns: GRID_COLS,
                      transform: `translateY(${virtualRow.start}px)`,
                    }}
                  >
                    <div className="px-3 py-1.5 font-mono text-[var(--color-text-secondary)]">
                      {row.file}
                    </div>
                    <div className="px-3 py-1.5 font-mono text-[var(--color-text-muted)]">
                      {row.section}
                    </div>
                    <div className="px-3 py-1.5 font-mono text-[var(--color-accent-hover)]">
                      {row.key}
                    </div>
                    <div
                      className="truncate px-3 py-1.5 text-[var(--color-text)]"
                      title={row.value}
                    >
                      {row.value}
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        )}
      </div>
      <p className="text-xs text-[var(--color-text-muted)]">{t("extra.readOnlyHint")}</p>
    </div>
  );
}
