import { AlertTriangle, Info, X } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import type { EditorPanel } from "@/lib/routing";
import { engineWarningAckKey } from "@/lib/routing";
import { DetailDialog } from "@/components/ds/DetailDialog";

const BASE_PANELS: EditorPanel[] = [
  "basic",
  "advanced",
  "input",
  "files",
  "presets",
  "backups",
];

interface Props {
  gameId: string;
  panel: EditorPanel;
  onPanelChange: (panel: EditorPanel) => void;
  engineStats: { total: number; on: number; off: number };
  showExtraTab?: boolean;
}

export function EditorModeBar({
  gameId,
  panel,
  onPanelChange,
  engineStats,
  showExtraTab = false,
}: Props) {
  const { t } = useTranslation("advanced");
  const [helpOpen, setHelpOpen] = useState(false);
  const [advancedWarningDismissed, setAdvancedWarningDismissed] =
    useState(false);

  const panels = useMemo(() => {
    if (showExtraTab || panel === "extra") {
      return [
        "basic",
        "advanced",
        "input",
        "files",
        "extra",
        "presets",
        "backups",
      ] as EditorPanel[];
    }
    return BASE_PANELS;
  }, [showExtraTab, panel]);

  useEffect(() => {
    try {
      setAdvancedWarningDismissed(
        sessionStorage.getItem(engineWarningAckKey(gameId)) === "1",
      );
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
    panel === "files"
      ? t("tabs.filesHint")
      : panel === "input"
        ? t("tabs.inputHint")
        : panel === "basic"
          ? t("tabs.basicHint")
          : panel === "advanced"
            ? t("tabs.advancedHint")
            : panel === "extra"
              ? t("tabs.extraHint")
              : panel === "presets"
                ? t("tabs.presetsHint")
                : t("tabs.backupsHint");
  const activeBody =
    panel === "files"
      ? t("mode.filesBody")
      : panel === "input"
        ? t("mode.inputBody")
        : panel === "basic"
          ? t("mode.basicBody")
          : panel === "advanced"
            ? t("mode.advancedBody")
            : panel === "extra"
              ? t("mode.extraBodyShort")
              : panel === "presets"
                ? t("mode.presetsBody")
                : t("mode.backupsBody");

  return (
    <div className="mb-4 shrink-0">
      <div className="flex items-center gap-2 border-b border-[var(--color-border)]">
        <div
          role="tablist"
          aria-label={t("tabs.aria")}
          className="material-tabs flex min-w-0 flex-1 overflow-x-auto"
        >
          {panels.map((value) => (
            <button
              key={value}
              type="button"
              role="tab"
              aria-selected={panel === value}
              tabIndex={panel === value ? 0 : -1}
              onClick={() => onPanelChange(value)}
              onKeyDown={(event) => {
                const index = panels.indexOf(value);
                const target =
                  event.key === "ArrowRight"
                    ? (index + 1) % panels.length
                    : event.key === "ArrowLeft"
                      ? (index + panels.length - 1) % panels.length
                      : event.key === "Home"
                        ? 0
                        : event.key === "End"
                          ? panels.length - 1
                          : -1;
                if (target < 0) return;
                event.preventDefault();
                onPanelChange(panels[target]);
                const tabs =
                  event.currentTarget.parentElement?.querySelectorAll<HTMLButtonElement>(
                    '[role="tab"]',
                  );
                tabs?.[target]?.focus();
              }}
            >
              {t(`tabs.${value}`)}
            </button>
          ))}
        </div>
        <button
          type="button"
          className="material-icon-button"
          aria-label={t("mode.help")}
          onClick={() => setHelpOpen(true)}
        >
          <Info size={19} />
        </button>
      </div>
      <DetailDialog
        open={helpOpen}
        title={activeHint}
        onClose={() => setHelpOpen(false)}
      >
        <p className="text-sm leading-relaxed text-[var(--color-text-secondary)]">
          {activeBody}
        </p>
        {(panel === "basic" || panel === "advanced") && (
          <p className="mt-3 text-sm">
            {t(panel === "basic" ? "mode.basicSafe" : "mode.advancedFiles")}
          </p>
        )}
      </DetailDialog>
      {panel === "advanced" && !advancedWarningDismissed && (
        <div className="mt-3 flex min-w-0 items-start gap-2 rounded-xl bg-[var(--color-warning-soft)] px-3 py-2 text-sm text-[var(--color-warning)]">
          <AlertTriangle size={16} className="mt-0.5 shrink-0" />
          <span className="min-w-0 flex-1 break-words">
            {t("engineWarning.inline")}{" "}
            {t("engineIni.short", {
              on: engineStats.on,
              total: engineStats.total,
            })}
          </span>
          <button
            type="button"
            onClick={dismissWarning}
            className="material-icon-button"
            aria-label={t("engineWarning.acknowledge")}
          >
            <X size={16} />
          </button>
        </div>
      )}
    </div>
  );
}
