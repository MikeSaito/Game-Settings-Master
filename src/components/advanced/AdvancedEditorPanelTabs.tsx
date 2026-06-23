import { useTranslation } from "react-i18next";
import type { EditorPanel } from "@/lib/routing";
import { Badge } from "@/components/ds/Badge";
import { cn } from "@/lib/core";

interface Props {
  panel: EditorPanel;
  onPanelChange: (panel: EditorPanel) => void;
}

export function AdvancedEditorPanelTabs({ panel, onPanelChange }: Props) {
  const { t } = useTranslation("advanced");

  const tabs: { id: EditorPanel; label: string; badge?: string }[] = [
    { id: "basic", label: t("tabs.basic") },
    {
      id: "advanced",
      label: t("tabs.advanced"),
      badge: t("tabs.engineBadge"),
    },
  ];

  return (
    <div
      className="inline-flex rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-hover)] p-1"
      role="tablist"
      aria-label={t("tabs.aria")}
    >
      {tabs.map(({ id, label, badge }) => (
        <button
          key={id}
          type="button"
          role="tab"
          aria-selected={panel === id}
          onClick={() => onPanelChange(id)}
          className={cn(
            "flex items-center gap-2 rounded-lg px-4 py-2 text-sm font-medium transition",
            panel === id
              ? "bg-[var(--color-bg)] text-[var(--color-text)] shadow-sm"
              : "text-muted hover:text-[var(--color-text-secondary)]",
          )}
        >
          {label}
          {badge && panel !== id && (
            <Badge tone="warning" className="text-[10px] px-1.5 py-0">
              {badge}
            </Badge>
          )}
        </button>
      ))}
    </div>
  );
}
