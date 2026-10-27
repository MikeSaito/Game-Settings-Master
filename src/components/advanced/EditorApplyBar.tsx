import { Save, Trash2, Zap } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { AdvancedEditorState } from "@/hooks/editor/useAdvancedEditorState";
import { canApplyPlan } from "@/lib/editor/validation";
import { Button } from "@/components/ds/Button";
import { DetailDialog } from "@/components/ds/DetailDialog";
import { Input } from "@/components/ds/Field";
import { useEffect, useState } from "react";

interface Props {
  state: AdvancedEditorState;
}

export function EditorApplyBar({ state }: Props) {
  const { t } = useTranslation("advanced");
  const { t: ti } = useTranslation("improvements");
  const [saveOpen, setSaveOpen] = useState(false);
  const [mode, setMode] = useState("changes");
  const [description, setDescription] = useState("");
  const [build, setBuild] = useState(state.game?.build_id ?? "");
  const [memory, setMemory] = useState("");
  const [vendor, setVendor] = useState("");
  const [rayTracing, setRayTracing] = useState(false);
  useEffect(() => {
    setDescription("");
    setMode("changes");
    setBuild(state.game?.build_id ?? "");
    setMemory("");
    setVendor("");
    setRayTracing(false);
  }, [state.game?.id]);
  const hasChanges = state.pendingChangesCount > 0;
  const applyLabel =
    state.panel === "basic" ? t("applyBasic") : t("applyAdvanced");
  const breakdown = state.pendingChangesBreakdown;
  const parts = [
    breakdown.sg > 0 ? t("changeBreakdown.sg", { count: breakdown.sg }) : null,
    breakdown.display > 0
      ? t("changeBreakdown.display", { count: breakdown.display })
      : null,
    breakdown.engine > 0
      ? t("changeBreakdown.engine", { count: breakdown.engine })
      : null,
  ].filter(Boolean);

  const {
    blockingErrors,
    needsWarningAck,
    allowed: validationAllowed,
  } = canApplyPlan(state.validationIssues, state.applyWarningsAcknowledged);
  const applyBlocked = state.gameRunning || !hasChanges || !validationAllowed;
  const saveBlocked =
    state.gameRunning ||
    (mode === "changes" && !hasChanges) ||
    blockingErrors ||
    (memory !== "" &&
      (!Number.isInteger(Number(memory)) || Number(memory) <= 0));

  return (
    <div className="flex min-h-0 flex-col overflow-y-auto border-t border-[var(--color-border)] pt-3">
      <div className="flex shrink-0 flex-wrap items-center gap-2">
        <div className="max-h-12 min-w-[100px] flex-1 overflow-y-auto">
          <div className="text-sm font-semibold text-[var(--color-text)]">
            {t("changesCount", { count: state.pendingChangesCount })}
          </div>
          {parts.length > 0 && (
            <div className="text-sm text-[var(--color-text-muted)]">
              {parts.join(" · ")}
            </div>
          )}
          {blockingErrors && (
            <div className="text-sm text-[var(--color-danger)]">
              {t("validation.applyBlocked")}
            </div>
          )}
          {needsWarningAck && (
            <div className="text-sm text-[var(--color-warning)]">
              {t("validation.confirmWarnings")}
            </div>
          )}
          {state.gameRunning && (
            <div className="text-sm text-[var(--color-warning)]">
              {t("gameRunningInline")}
            </div>
          )}
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
          icon={<Save size={15} />}
          onClick={() => setSaveOpen(true)}
          disabled={state.gameRunning}
        >
          {t("savePreset")}
        </Button>
        <Button
          variant="primary"
          size="sm"
          icon={<Zap size={15} />}
          onClick={() => state.applyCustomMutation.mutate()}
          loading={state.applyCustomMutation.isPending}
          disabled={applyBlocked}
          title={
            blockingErrors
              ? t("validation.applyBlocked")
              : needsWarningAck
                ? t("validation.confirmWarnings")
                : undefined
          }
        >
          {applyLabel}
        </Button>
      </div>
      <DetailDialog
        open={saveOpen}
        title={ti("presets.options")}
        onClose={() => {
          if (!state.saveOverrideMutation.isPending) setSaveOpen(false);
        }}
        footer={
          <>
            <Button
              variant="secondary"
              icon={<Save size={15} />}
              onClick={() =>
                state.saveOverrideMutation.mutate(
                  {
                    format_version: 2,
                    mode,
                    description,
                    source_game_id: state.game!.id,
                    source_game_name: state.game!.name,
                    game_build: build || null,
                    engine_family: state.game!.engine_family ?? null,
                    engine_version: state.game!.engine_version ?? null,
                    gpu_vendor: vendor || null,
                    min_dedicated_memory_mb: memory ? Number(memory) : null,
                    requires_ray_tracing: rayTracing,
                  },
                  { onSuccess: () => setSaveOpen(false) },
                )
              }
              loading={state.saveOverrideMutation.isPending}
              disabled={saveBlocked}
              title={blockingErrors ? t("validation.applyBlocked") : undefined}
            >
              {t("savePreset")}
            </Button>
          </>
        }
      >
        {state.saveOverrideMutation.error && (
          <p role="alert" className="mb-3 text-sm text-[var(--color-danger)]">
            {String(state.saveOverrideMutation.error)}
          </p>
        )}
        <div className="mb-4">
          <Input
            label={t("presetNameLabel")}
            value={state.overrideName}
            onChange={(event) => state.setOverrideName(event.target.value)}
          />
        </div>
        <div className="mt-2 grid gap-2 sm:grid-cols-2">
          <label>
            {ti("presets.mode")}{" "}
            <select
              value={mode}
              onChange={(event) => setMode(event.target.value)}
              className="mt-2 block w-full rounded-xl bg-[var(--color-bg-soft)] p-2"
            >
              <option value="changes">{ti("presets.changes")}</option>
              <option value="profile">{ti("presets.profile")}</option>
            </select>
          </label>
          <div>
            <Input
              label={ti("presets.description")}
              value={description}
              maxLength={4096}
              onChange={(event) => setDescription(event.target.value)}
            />
          </div>
          <div>
            <Input
              label={ti("presets.build")}
              value={build}
              maxLength={120}
              onChange={(event) => setBuild(event.target.value)}
            />
          </div>
          <div>
            <Input
              label={ti("presets.memory")}
              type="number"
              min="1"
              value={memory}
              onChange={(event) => setMemory(event.target.value)}
            />
          </div>
          <label>
            {ti("presets.vendor")}{" "}
            <select
              value={vendor}
              onChange={(event) => setVendor(event.target.value)}
              className="mt-2 block w-full rounded-xl bg-[var(--color-bg-soft)] p-2"
            >
              <option value="">{ti("presets.anyGpu")}</option>
              <option value="nvidia">NVIDIA</option>
              <option value="amd">AMD</option>
              <option value="intel">Intel</option>
            </select>
          </label>
          <label>
            <input
              type="checkbox"
              checked={rayTracing}
              onChange={(event) => setRayTracing(event.target.checked)}
            />{" "}
            {ti("presets.requiresRt")}
          </label>
        </div>
      </DetailDialog>
    </div>
  );
}
