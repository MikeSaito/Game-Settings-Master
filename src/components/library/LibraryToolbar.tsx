import { Grid2X2, List, Plus, RefreshCw, Search } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ds/Button";
import { Input } from "@/components/ds/Field";
import { SegmentControl } from "@/components/ds/SegmentControl";

export type LibraryViewMode = "grid" | "list";

interface Props {
  query: string;
  onQueryChange: (value: string) => void;
  viewMode: LibraryViewMode;
  onViewModeChange: (value: LibraryViewMode) => void;
  onScan: () => void;
  scanning: boolean;
  onAdd: () => void;
  adding: boolean;
}

export function LibraryToolbar({
  query,
  onQueryChange,
  viewMode,
  onViewModeChange,
  onScan,
  scanning,
  onAdd,
  adding,
}: Props) {
  const { t } = useTranslation("library");

  return (
    <div className="flex flex-wrap items-center gap-2 rounded-[var(--radius-panel)] border border-[var(--color-border)] bg-[var(--color-surface)] p-2">
      <div className="min-w-[260px] flex-1">
        <Input
          icon={<Search size={15} />}
          placeholder={t("search.placeholder")}
          value={query}
          onChange={(event) => onQueryChange(event.target.value)}
          aria-label={t("search.aria")}
        />
      </div>
      <SegmentControl
        value={viewMode}
        onChange={onViewModeChange}
        ariaLabel={t("viewMode")}
        options={[
          { value: "grid", label: <Grid2X2 size={15} aria-label={t("view.grid")} /> },
          { value: "list", label: <List size={15} aria-label={t("view.list")} /> },
        ]}
      />
      <Button
        variant="secondary"
        icon={<RefreshCw size={15} className={scanning ? "animate-spin" : undefined} />}
        onClick={onScan}
        loading={scanning}
      >
        {t("actions.scan")}
      </Button>
      <Button variant="primary" icon={<Plus size={15} />} onClick={onAdd} loading={adding}>
        {t("actions.add")}
      </Button>
    </div>
  );
}
