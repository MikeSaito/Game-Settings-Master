import { Save, Trash2, Zap } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { AdvancedEditorState } from "@/hooks/editor/useAdvancedEditorState";
import { canApplyPlan } from "@/lib/editor/validation";
import { Button } from "@/components/ds/Button";
import { Input } from "@/components/ds/Field";
import { useEffect, useState } from "react";

interface Props {
  state: AdvancedEditorState;
}

export function EditorApplyBar({ state }: Props) {
  const { t } = useTranslation("advanced");
  const { t: ti } = useTranslation("improvements");
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
    <div className="flex min-h-0 flex-col overflow-y-auto rounded-[var(--radius-panel)] border border-[var(--color-border)] bg-[var(--color-surface)] p-2 shadow-[var(--shadow-panel)]">
      <details className="mb-2 min-h-8 overflow-y-auto overscroll-contain text-sm">
        <summary>{ti("presets.options")}</summary>
        <div className="mt-2 grid gap-2 sm:grid-cols-2">
          <label>
            {ti("presets.mode")}{" "}
            <select
              value={mode}
              onChange={(event) => setMode(event.target.value)}
              className="rounded bg-[var(--color-bg-soft)] p-2"
            >
              <option value="changes">{ti("presets.changes")}</option>
              <option value="profile">{ti("presets.profile")}</option>
            </select>
          </label>
          <label>
            {ti("presets.description")}
            <Input
              value={description}
              maxLength={4096}
              onChange={(event) => setDescription(event.target.value)}
            />
          </label>
          <label>
            {ti("presets.build")}
            <Input
              value={build}
              maxLength={120}
              onChange={(event) => setBuild(event.target.value)}
            />
          </label>
          <label>
            {ti("presets.memory")}
            <Input
              type="number"
              min="1"
              value={memory}
              onChange={(event) => setMemory(event.target.value)}
            />
          </label>
          <label>
            {ti("presets.vendor")}{" "}
            <select
              value={vendor}
              onChange={(event) => setVendor(event.target.value)}
              className="rounded bg-[var(--color-bg-soft)] p-2"
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
      </details>
      <div className="flex shrink-0 flex-wrap items-center gap-2">
        <div className="max-h-12 min-w-[160px] flex-1 overflow-y-auto">
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
          onClick={() =>
            state.saveOverrideMutation.mutate({
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
            })
          }
          loading={state.saveOverrideMutation.isPending}
          disabled={saveBlocked}
          title={blockingErrors ? t("validation.applyBlocked") : undefined}
        >
          {t("savePreset")}
        </Button>
        <Button
          variant="primary"
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
    </div>
  );
}
