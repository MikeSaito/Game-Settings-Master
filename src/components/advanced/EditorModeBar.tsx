import { AlertTriangle, X } from "lucide-react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import type { EditorPanel } from "@/lib/routing";
import { engineWarningAckKey } from "@/lib/routing";
import { cn } from "@/lib/core";
import { SegmentControl } from "@/components/ds/SegmentControl";

const PANELS: EditorPanel[] = ["basic", "advanced", "backups"];

interface Props {
  gameId: string;
  panel: EditorPanel;
  onPanelChange: (panel: EditorPanel) => void;
  engineStats: { total: number; on: number; off: number };
}

export function EditorModeBar({
  gameId,
  panel,
  onPanelChange,
  engineStats,
}: Props) {
  const { t } = useTranslation("advanced");
  const [advancedWarningDismissed, setAdvancedWarningDismissed] = useState(false);

  useEffect(() => {
    try {
      setAdvancedWarningDismissed(sessionStorage.getItem(engineWarningAckKey(gameId)) === "1");
    } catch {
      setAdvancedWarningDismissed(false);
    }
  }, [gameId]);

  const dismissWarning = () => {
    setAdvancedWarningDismissed(true);
    try {
      sessionStorage.setItem(engineWarningAckKey(gameId), "1");
    } catch {
      /* ignore */
    }
  };

  const activeHint =
    panel === "basic"
      ? t("tabs.basicHint")
      : panel === "advanced"
        ? t("tabs.advancedHint")
        : t("tabs.backupsHint");
  const activeBody =
    panel === "basic"
      ? t("mode.basicBody")
      : panel === "advanced"
        ? t("mode.advancedBody")
        : t("mode.backupsBody");

  return (
    <div className="mb-3 rounded-[var(--radius-panel)] border border-[var(--color-border)] bg-[var(--color-surface)] p-3 shadow-[var(--shadow-soft)]">
      <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
        <div className="min-w-0 flex-1">
          <SegmentControl
            value={panel}
            onChange={onPanelChange}
            ariaLabel={t("tabs.aria")}
            segmentClassName="min-w-[5.5rem] px-3 sm:min-w-[6.75rem]"
            options={PANELS.map((value) => ({
              value,
              label: t(`tabs.${value}`),
            }))}
          />
          <div className="mt-2">
            <div className="text-sm font-medium text-[var(--color-text)]">{activeHint}</div>
            <div className="text-xs text-[var(--color-text-muted)]">{activeBody}</div>
          </div>
        </div>

        {panel !== "backups" && (
          <div
            className={cn(
              "min-w-0 rounded-[var(--radius-control)] border px-3 py-2 text-xs sm:max-w-[min(50%,34rem)]",
              panel === "basic"
                ? "border-[var(--color-success)]/25 bg-[var(--color-success-soft)] text-[var(--color-text-secondary)]"
                : "border-[var(--color-warning)]/35 bg-[var(--color-warning-soft)] text-[var(--color-warning)]",
            )}
          >
            {panel === "basic" ? (
              t("mode.basicSafe")
            ) : advancedWarningDismissed ? (
              t("mode.advancedFiles")
            ) : (
              <div className="flex min-w-0 gap-2">
                <AlertTriangle size={14} className="mt-0.5 shrink-0" />
                <span className="min-w-0 break-words leading-relaxed">
                  {t("engineWarning.inline")}{" "}
                  {t("engineIni.short", { on: engineStats.on, total: engineStats.total })}
                </span>
                <button
                  type="button"
                  onClick={dismissWarning}
                  className="ml-1 grid h-5 w-5 shrink-0 place-items-center rounded hover:bg-[var(--color-surface-hover)]"
                  aria-label={t("engineWarning.acknowledge")}
                >
                  <X size={13} />
                </button>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
