import { Search } from "lucide-react";
import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import type { EditorFilterMode } from "@/lib/routing";
import { Input } from "../ds/Field";
import { Panel } from "../ds/Panel";
import { SegmentControl } from "../ds/SegmentControl";
import { cn } from "@/lib/core";

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
  filterMode: EditorFilterMode;
  onFilterModeChange: (mode: EditorFilterMode) => void;
}

const FILTER_MODES: EditorFilterMode[] = ["recommended", "full", "ini_only"];

export function EditorSidebar({
  search,
  onSearchChange,
  categories,
  activeCategory,
  onCategoryChange,
  filterMode,
  onFilterModeChange,
}: Props) {
  const { t } = useTranslation("advanced");

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement | null;
      const editing = target?.tagName === "INPUT" || target?.tagName === "TEXTAREA" || target?.tagName === "SELECT";
      if ((event.key === "/" || ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k")) && !editing) {
        event.preventDefault();
        document.getElementById("advanced-search")?.focus();
      }
      if (event.key === "Escape" && search) onSearchChange("");
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [onSearchChange, search]);

  return (
    <Panel className="w-60 shrink-0 self-start overflow-hidden p-3 lg:sticky lg:top-4 lg:max-h-[calc(100dvh-11rem)]">
      <div className="space-y-3">
        <Input
          id="advanced-search"
          icon={<Search size={15} />}
          placeholder={t("searchPlaceholder")}
          value={search}
          onChange={(event) => onSearchChange(event.target.value)}
        />

        <SegmentControl
          value={filterMode}
          onChange={onFilterModeChange}
          options={FILTER_MODES.map((mode) => ({
            value: mode,
            label: t(`filter.${mode}`),
          }))}
          className="w-full flex-col gap-1"
          segmentClassName="w-full whitespace-normal px-2 py-1.5 text-[11px] leading-tight"
        />
      </div>

      <nav className="mt-3 h-[calc(100%-120px)] overflow-y-auto pr-1" aria-label={t("categoryNav")}>
        {categories.map(({ cat, count }) => {
          const active = activeCategory === cat;
          return (
            <button
              key={cat}
              type="button"
              onClick={() => onCategoryChange(cat)}
              className={cn(
                "mb-1 flex w-full items-center justify-between rounded-[var(--radius-control)] px-2.5 py-2 text-left text-sm transition",
                active
                  ? "bg-[var(--color-accent-soft)] text-[var(--color-text)]"
                  : "text-[var(--color-text-muted)] hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text-secondary)]",
              )}
            >
              <span className="truncate">{t(`category.${cat}`, { defaultValue: cat })}</span>
              <span className="ml-2 rounded bg-[var(--color-bg)] px-1.5 py-0.5 font-mono text-[11px] text-[var(--color-text-faint)]">
                {count}
              </span>
            </button>
          );
        })}
      </nav>
    </Panel>
  );
}
