import { ArrowRight, Equal, Plus } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { ConfigDiffEntry } from "@/lib/core";
import { Card } from "../ui/Card";
import { EmptyState } from "../ui/EmptyState";

interface Props {
  diff: ConfigDiffEntry[];
  loading?: boolean;
}

export function IniDiffView({ diff, loading }: Props) {
  const { t } = useTranslation("advanced");
  if (loading) {
    return (
      <Card padding="lg" hover={false}>
        <div className="flex items-center gap-3 text-body">
          <span className="h-5 w-5 animate-spin rounded-full border-2 border-[var(--color-border)] border-t-[var(--color-accent)]" />
          {t("loadingPreview")}
        </div>
      </Card>
    );
  }

  if (diff.length === 0) {
    return (
      <EmptyState
        icon={Equal}
        title={t("noDiff.title")}
        description={t("noDiff.desc")}
        className="py-10"
      />
    );
  }

  return (
    <Card padding="sm" hover={false} className="overflow-hidden p-0">
      <div className="max-h-80 overflow-auto">
        <table className="w-full text-sm">
          <thead className="sticky top-0 z-10 bg-[var(--color-bg-elevated)] text-left">
            <tr className="border-b border-[var(--color-border)] text-xs font-semibold uppercase tracking-wide text-muted">
              <th className="px-4 py-3">{t("table.file")}</th>
              <th className="px-4 py-3">{t("table.param")}</th>
              <th className="px-4 py-3">{t("table.was")}</th>
              <th className="w-8 px-2 py-3" />
              <th className="px-4 py-3">{t("table.willBe")}</th>
            </tr>
          </thead>
          <tbody>
            {diff.map((entry, i) => (
              <tr
                key={`${entry.file}-${entry.key}-${i}`}
                className="border-b border-[var(--color-border)]/60 hover:bg-[var(--color-bg-hover)]"
              >
                <td className="px-4 py-2.5 text-sm text-muted">{entry.file}</td>
                <td className="px-4 py-2.5">
                  <code className="font-mono text-xs text-code">{entry.key}</code>
                  <div className="mt-0.5 font-mono text-xs text-faint">[{entry.section}]</div>
                </td>
                <td className="px-4 py-2.5 font-mono text-sm text-[#f0a0a0]">
                  {entry.old_value ?? "—"}
                </td>
                <td className="px-2 py-2.5 text-faint">
                  <ArrowRight size={14} />
                </td>
                <td className="px-4 py-2.5 font-mono text-sm text-[#8fd9a8]">
                  {entry.new_value}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <div className="flex items-center gap-2 border-t border-[var(--color-border)] px-4 py-2.5 text-sm text-muted">
        <Plus size={12} className="text-[#8fd9a8]" />
        {t("diffCount", { count: diff.length })}
      </div>
    </Card>
  );
}
