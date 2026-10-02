import { Download, Trash2, Upload, Zap } from "lucide-react";
import { useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { ApplyValidationPanel } from "@/components/advanced/ApplyValidationPanel";
import type { AdvancedEditorState } from "@/hooks/editor/useAdvancedEditorState";
import { canApplyPlan } from "@/lib/editor/validation";
import type { GameOverride } from "@/lib/core";
import { Button } from "@/components/ds/Button";
import {
  parsePreset,
  presetCompatibility,
} from "@/lib/editor/presetCompatibility";

interface Props {
  state: AdvancedEditorState;
  variant?: "embedded" | "page";
}

function downloadPresetJson(override: GameOverride) {
  const payload = {
    ...override,
    files: override.files,
    removals: override.removals ?? {},
  };
  const blob = new Blob([JSON.stringify(payload, null, 2)], {
    type: "application/json",
  });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = `${override.name.replace(/[^\w.-]+/g, "_") || "preset"}.uesm-preset.json`;
  anchor.click();
  URL.revokeObjectURL(url);
}

export function SavedPresetsPanel({ state, variant = "embedded" }: Props) {
  const { t } = useTranslation("advanced");
  const { t: ti } = useTranslation("improvements");
  const [imported, setImported] = useState<GameOverride>();
  const [importAck, setImportAck] = useState(false);
  const [legacyImport, setLegacyImport] = useState(false);
  const importRef = useRef<HTMLInputElement>(null);
  const pending = state.pendingPresetApply;
  const presetGate = canApplyPlan(
    state.presetApplyIssues,
    state.presetApplyWarningsAcknowledged,
  );

  const onImportFile = async (file: File) => {
    if (!state.game?.id) return;
    try {
      const text = await file.text();
      const preset = parsePreset(text, state.game.id);
      if (!preset) {
        state.setApplyError(t("presets.importInvalid"));
        return;
      }
      setImported(preset);
      setImportAck(false);
      setLegacyImport(!JSON.parse(text).metadata);
    } catch {
      state.setApplyError(t("presets.importInvalid"));
    }
  };

  const importWarnings =
    imported && state.game
      ? [
          ...presetCompatibility(imported, state.game, state.gpu),
          ...(legacyImport ? [ti("presets.legacy")] : []),
        ]
      : [];

  return (
    <section
      className={
        variant === "page"
          ? "rounded-[var(--radius-panel)] border border-[var(--color-border)] bg-[var(--color-surface)] p-4"
          : "mt-3 rounded-[var(--radius-panel)] border border-[var(--color-border)] bg-[var(--color-surface)] p-3"
      }
    >
      <div className="mb-2 flex flex-wrap items-center justify-between gap-2">
        <h3 className="text-sm font-semibold text-[var(--color-text)]">
          {t("savedPresets")}
        </h3>
        <div className="flex gap-2">
          <Button
            variant="ghost"
            className="!py-1 !px-2 text-xs"
            icon={<Upload size={14} />}
            onClick={() => importRef.current?.click()}
            loading={state.importOverrideMutation.isPending}
          >
            {t("presets.import")}
          </Button>
          <input
            ref={importRef}
            type="file"
            accept="application/json,.json"
            className="hidden"
            onChange={(event) => {
              const file = event.target.files?.[0];
              event.target.value = "";
              if (file) void onImportFile(file);
            }}
          />
        </div>
      </div>

      {pending && (
        <div className="mb-3 space-y-2 rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-soft)] p-3">
          <div className="text-sm font-medium text-[var(--color-text)]">
            {t("presets.applyConfirmTitle", { name: pending.name })}
          </div>
          <ApplyValidationPanel
            issues={state.presetApplyIssues}
            warningsAcknowledged={state.presetApplyWarningsAcknowledged}
            onWarningsAcknowledgedChange={
              state.setPresetApplyWarningsAcknowledged
            }
          />
          <div className="flex flex-wrap gap-2">
            <Button
              variant="ghost"
              className="!py-1 !px-2 text-xs"
              onClick={state.cancelPendingPresetApply}
            >
              {t("presets.applyCancel")}
            </Button>
            <Button
              variant="primary"
              className="!py-1 !px-2 text-xs"
              icon={<Zap size={14} />}
              onClick={state.confirmPendingPresetApply}
              disabled={!presetGate.allowed || state.gameRunning}
              loading={state.applyOverrideMutation.isPending}
            >
              {t("presets.applyConfirm")}
            </Button>
          </div>
        </div>
      )}

      {imported && state.game && imported.game_id === state.game.id && (
        <div className="my-3 space-y-2 rounded border border-[var(--color-border)] p-3">
          <h4>{ti("presets.importPreview", { name: imported.name })}</h4>
          <p>{imported.metadata?.description}</p>
          {importWarnings.map((warning) => (
            <p key={warning} className="text-[var(--color-warning)]">
              {warning}
            </p>
          ))}
          {importWarnings.length > 0 && (
            <label>
              <input
                type="checkbox"
                checked={importAck}
                onChange={(event) => setImportAck(event.target.checked)}
              />{" "}
              {ti("preview.ack")}
            </label>
          )}
          <div className="flex gap-2">
            <Button variant="ghost" onClick={() => setImported(undefined)}>
              {ti("cancel")}
            </Button>
            <Button
              disabled={importWarnings.length > 0 && !importAck}
              loading={state.importOverrideMutation.isPending}
              onClick={() =>
                void state.importOverrideMutation
                  .mutateAsync(imported)
                  .then(() => setImported(undefined))
                  .catch(() => {})
              }
            >
              {ti("presets.confirmImport")}
            </Button>
          </div>
        </div>
      )}

      <ul className="space-y-1.5">
        {state.overrides.length === 0 && (
          <li className="px-1 py-2 text-xs text-[var(--color-text-muted)]">
            {t("presets.empty")}
          </li>
        )}
        {state.overrides.map((override) => (
          <li
            key={`${override.game_id}-${override.name}`}
            className="flex items-center justify-between gap-3 rounded-lg border border-[var(--color-border)] px-3 py-2"
          >
            <span className="min-w-0 truncate text-sm text-[var(--color-text-secondary)]">
              {override.name}
              {override.metadata?.description && (
                <small className="block whitespace-normal text-[var(--color-text-muted)]">
                  {override.metadata.description}
                </small>
              )}
            </span>
            <div className="flex shrink-0 gap-1">
              <Button
                variant="ghost"
                className="!py-1 !px-2 text-xs"
                icon={<Download size={14} />}
                onClick={() => downloadPresetJson(override)}
                title={t("presets.export")}
              >
                {t("presets.export")}
              </Button>
              <Button
                variant="secondary"
                className="!py-1 !px-2 text-xs"
                icon={<Zap size={14} />}
                onClick={() => state.requestApplyPreset(override)}
                disabled={
                  state.gameRunning ||
                  state.applyOverrideMutation.isPending ||
                  (pending != null && pending.name !== override.name)
                }
                loading={state.applyingPresetName === override.name}
              >
                {t("apply")}
              </Button>
              <button
                type="button"
                onClick={() =>
                  state.deleteOverrideMutation.mutate({
                    gameId: override.game_id,
                    name: override.name,
                  })
                }
                className="rounded-lg p-1.5 text-[var(--color-text-muted)] transition hover:bg-[#2e1a1a] hover:text-[#f08080]"
                title={t("presets.delete")}
              >
                <Trash2 size={15} />
              </button>
            </div>
          </li>
        ))}
      </ul>
    </section>
  );
}
