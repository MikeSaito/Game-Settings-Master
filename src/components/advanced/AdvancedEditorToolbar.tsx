import { Search } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Alert } from "../ui/Alert";
import { Input } from "../ui/Input";
import { cn } from "../../lib/cn";

interface CategoryItem {
  cat: string;
  count: number;
}

interface Props {
  search: string;
  onSearchChange: (value: string) => void;
  categories: CategoryItem[];
  activeCategory: string;
  onCategoryChange: (category: string) => void;
  showEngineIniHint: boolean;
  engineStats: { total: number; on: number; off: number };
}

export function AdvancedEditorToolbar({
  search,
  onSearchChange,
  categories,
  activeCategory,
  onCategoryChange,
  showEngineIniHint,
  engineStats,
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

      {showEngineIniHint && (
        <Alert tone="info" title={t("engineIni.title")}>
          {t("engineIni.before")}
          <strong>{t("engineIni.onOff")}</strong>
          {t("engineIni.after", {
            on: engineStats.on,
            total: engineStats.total,
          })}
        </Alert>
      )}
    </>
  );
}
