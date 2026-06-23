import { Search } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { AdvancedPanel } from "@/lib/routing";
import { Alert } from "@/components/ds/Feedback";
import { Input } from "@/components/ds/Field";
import { Toggle } from "@/components/ds/Toggle";
import { cn } from "@/lib/core";

interface CategoryItem {
  cat: string;
  count: number;
}

interface Props {
  panel: AdvancedPanel;
  search: string;
  onSearchChange: (value: string) => void;
  categories: CategoryItem[];
  activeCategory: string;
  onCategoryChange: (category: string) => void;
  showEngineIniHint: boolean;
  engineStats: { total: number; on: number; off: number };
  showRecommendedOnly: boolean;
  onShowRecommendedOnlyChange: (value: boolean) => void;
}

export function AdvancedEditorToolbar({
  panel,
  search,
  onSearchChange,
  categories,
  activeCategory,
  onCategoryChange,
  showEngineIniHint,
  engineStats,
  showRecommendedOnly,
  onShowRecommendedOnlyChange,
}: Props) {
  const { t } = useTranslation("advanced");

  return (
    <>
      <Input
        icon={<Search size={16} />}
        placeholder={t("searchPlaceholder")}
        value={search}
        onChange={(e) => onSearchChange(e.target.value)}
      />

      <label className="flex cursor-pointer items-center justify-between gap-3 rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-hover)] px-3 py-2 text-sm">
        <span className="text-muted">
          {showRecommendedOnly ? t("filter.recommended") : t("filter.allInIni")}
        </span>
        <Toggle
          checked={showRecommendedOnly}
          onChange={onShowRecommendedOnlyChange}
          aria-label={t("filter.recommended")}
        />
      </label>

      <div className="flex flex-wrap gap-1.5">
        {categories.map(({ cat, count }) => (
          <button
            key={cat}
            type="button"
            onClick={() => onCategoryChange(cat)}
            className={cn(
              "rounded-xl px-3 py-2 text-sm font-medium transition",
              activeCategory === cat
                ? "bg-[var(--color-bg-active)] text-[var(--color-text)] ring-1 ring-[var(--color-accent)]/40"
                : "text-muted hover:bg-[var(--color-bg-hover)] hover:text-[var(--color-text-secondary)]",
            )}
          >
            {t(`category.${cat}`, { defaultValue: cat })}
            <span className="ml-1.5 text-xs opacity-60">{count}</span>
          </button>
        ))}
      </div>

      {panel === "advanced" && showEngineIniHint && (
        <Alert tone="info" title={t("engineIni.title")}>
          {t("engineIni.before", {
            on: engineStats.on,
            total: engineStats.total,
          })}
        </Alert>
      )}
    </>
  );
}
