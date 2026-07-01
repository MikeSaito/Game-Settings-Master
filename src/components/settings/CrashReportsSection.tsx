import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";
import { clearCrashReports, listCrashReports } from "@/lib/api";
import type { CrashReportEntry } from "@/lib/crashReport";
import { copyCrashReport, openCrashReportIssue } from "@/lib/crashReport";
import { Button } from "@/components/ds/Button";

function formatReportTime(createdAt: string): string {
  const date = new Date(createdAt);
  if (Number.isNaN(date.getTime())) return createdAt;
  return date.toLocaleString();
}

function CrashReportRow({
  report,
  kindLabel,
}: {
  report: CrashReportEntry;
  kindLabel: string;
}) {
  const { t } = useTranslation("settings");

  return (
    <li className="rounded-[var(--radius-control)] border border-[var(--color-border)] bg-[var(--color-bg-soft)] p-3">
      <div className="flex flex-wrap items-start justify-between gap-2">
        <div className="min-w-0 flex-1 space-y-1">
          <p className="truncate text-sm font-medium text-[var(--color-text)]">
            {report.message}
          </p>
          <p className="text-xs text-[var(--color-text-muted)]">
            {kindLabel} · {formatReportTime(report.created_at)} · v{report.app_version}
          </p>
        </div>
        <div className="flex shrink-0 flex-wrap gap-2">
          <Button variant="secondary" size="sm" onClick={() => openCrashReportIssue(report)}>
            {t("crashReports.openGithub")}
          </Button>
          <Button variant="ghost" size="sm" onClick={() => void copyCrashReport(report)}>
            {t("crashReports.copyReport")}
          </Button>
        </div>
      </div>
    </li>
  );
}

export function CrashReportsSection({
  enabled,
  onEnabledChange,
}: {
  enabled: boolean;
  onEnabledChange: (enabled: boolean) => void;
}) {
  const { t } = useTranslation("settings");
  const queryClient = useQueryClient();
  const { data: reports = [] } = useQuery({
    queryKey: ["crash-reports"],
    queryFn: listCrashReports,
    enabled,
    staleTime: 30_000,
  });

  const kindLabel = (kind: CrashReportEntry["kind"]) =>
    t(`crashReports.kind.${kind}`, { defaultValue: kind });

  const handleClear = async () => {
    await clearCrashReports();
    await queryClient.invalidateQueries({ queryKey: ["crash-reports"] });
  };

  return (
    <section className="space-y-3">
      <h3 className="text-xs font-semibold uppercase tracking-wide text-[var(--color-text-muted)]">
        {t("crashReports.title")}
      </h3>
      <label className="flex items-start justify-between gap-3 rounded-[var(--radius-control)] border border-[var(--color-border-strong)] bg-[var(--color-bg-soft)] p-3">
        <span className="min-w-0 flex-1">
          <span className="block text-sm font-medium text-[var(--color-text)]">
            {t("crashReports.enable")}
          </span>
          <span className="mt-0.5 block text-xs text-[var(--color-text-muted)]">
            {t("crashReports.enableDesc")}
          </span>
        </span>
        <input
          type="checkbox"
          checked={enabled}
          onChange={(event) => onEnabledChange(event.target.checked)}
          className="mt-1 h-4 w-4 accent-[var(--color-accent)]"
          aria-label={t("crashReports.enable")}
        />
      </label>
      {enabled && reports.length > 0 && (
        <div className="space-y-3 rounded-[var(--radius-control)] border border-[var(--color-border)] bg-[var(--color-bg-soft)] p-3">
          <p className="text-xs text-[var(--color-text-muted)]">
            {t("crashReports.pending", { count: reports.length })}
          </p>
          <ul className="space-y-2" aria-label={t("crashReports.listLabel")}>
            {reports.map((report) => (
              <CrashReportRow
                key={report.id}
                report={report}
                kindLabel={kindLabel(report.kind)}
              />
            ))}
          </ul>
          <Button variant="ghost" onClick={() => void handleClear()}>
            {t("crashReports.clear")}
          </Button>
        </div>
      )}
    </section>
  );
}
