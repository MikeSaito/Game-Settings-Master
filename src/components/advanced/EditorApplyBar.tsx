import { Save, Trash2, Zap } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { AdvancedEditorState } from "@/hooks/editor/useAdvancedEditorState";
import { Button } from "@/components/ds/Button";
import { Input } from "@/components/ds/Field";

interface Props {
  state: AdvancedEditorState;
}

export function EditorApplyBar({ state }: Props) {
  const { t } = useTranslation("advanced");
  const hasChanges = state.pendingChangesCount > 0;
  const applyLabel =
    state.panel === "basic" ? t("applyBasic") : t("applyAdvanced");
  const breakdown = state.pendingChangesBreakdown;
  const parts = [
    breakdown.sg > 0 ? t("changeBreakdown.sg", { count: breakdown.sg }) : null,
    breakdown.display > 0 ? t("changeBreakdown.display", { count: breakdown.display }) : null,
    breakdown.engine > 0 ? t("changeBreakdown.engine", { count: breakdown.engine }) : null,
  ].filter(Boolean);

  return (
    <div className="sticky bottom-3 z-10 mt-3 rounded-[var(--radius-panel)] border border-[var(--color-border)] bg-[var(--color-surface)] p-2 shadow-[var(--shadow-panel)]">
      <div className="flex flex-wrap items-center gap-2">
        <div className="min-w-[160px] flex-1">
          <div className="text-xs font-semibold text-[var(--color-text)]">
            {t("changesCount", { count: state.pendingChangesCount })}
          </div>
          {parts.length > 0 && (
            <div className="text-xs text-[var(--color-text-muted)]">{parts.join(" · ")}</div>
          )}
          {state.gameRunning && (
            <div className="text-xs text-[var(--color-warning)]">{t("gameRunningInline")}</div>
          )}
        </div>
        <div className="w-48">
          <Input
            aria-label={t("presetNameLabel")}
            value={state.overrideName}
            onChange={(event) => state.setOverrideName(event.target.value)}
          />
        </div>
        <Button
          variant="ghost"
          icon={<Trash2 size={15} />}
          onClick={state.discardChanges}
          disabled={!hasChanges}
        >
          {t("discard")}
        </Button>
        <Button
          variant="secondary"
          icon={<Save size={15} />}
          onClick={() => state.saveOverrideMutation.mutate()}
          loading={state.saveOverrideMutation.isPending}
          disabled={!hasChanges}
        >
          {t("savePreset")}
        </Button>
        <Button
          variant="primary"
          icon={<Zap size={15} />}
          onClick={() => state.applyCustomMutation.mutate()}
          loading={state.applyCustomMutation.isPending}
          disabled={state.gameRunning || !hasChanges}
        >
          {applyLabel}
        </Button>
      </div>
    </div>
  );
}
