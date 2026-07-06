import { useMutation, useQueryClient } from "@tanstack/react-query";
import type { MutableRefObject } from "react";
import {
  applyCustom,
  applyGameOverride,
  deleteGameOverride,
  saveGameOverride,
} from "@/lib/api";
import { buildCustomChanges } from "@/lib/editor";
import { assertApplyPlanAllowed, assertPresetStorable, validateOverridePlan } from "@/lib/editor/validation";
import type { ValidationIssue } from "@/lib/editor/validation";
import { invalidateGameWorkspace } from "@/lib/game/invalidateGameWorkspace";
import type { EditorPanel } from "@/lib/routing";
import { formatInvokeError } from "@/lib/core";
import type { GameOverride, GameParameter, GameProfile, GpuCapabilities, ScalabilityLimits } from "@/lib/core";

export interface ApplyOverrideInput {
  override: GameOverride;
  warningsAcknowledged: boolean;
}

interface Options {
  game: GameProfile | null;
  configDir: string;
  runningExeName: string | null;
  params: GameParameter[];
  parameters: GameParameter[];
  panel: EditorPanel;
  gpu: GpuCapabilities | undefined;
  limits: ScalabilityLimits | undefined;
  limitsPending: boolean;
  gpuPending: boolean;
  gpuUnavailable: boolean;
  engineEnabled: Set<string>;
  editableCategories: Set<string>;
  shippedIniKeys: ReadonlySet<string>;
  overrideName: string;
  validationIssues: ValidationIssue[];
  applyWarningsAcknowledged: boolean;
  activeGameIdRef: MutableRefObject<string | undefined>;
  setMessage: (message: string | undefined) => void;
  setApplyError: (error: string | undefined) => void;
  onApplied: () => void;
  onPresetApplied?: () => void;
  onPresetApplyStart?: (name: string) => void;
  onPresetApplyEnd?: () => void;
  t: (key: string, options?: Record<string, unknown>) => string;
}

function buildChanges(options: Options) {
  return buildCustomChanges(
    options.params,
    options.parameters,
    options.gpu,
    options.engineEnabled,
    options.editableCategories,
    options.panel,
    options.shippedIniKeys,
  );
}

function overrideValidationContext(options: Options) {
  if (!options.game) return null;
  return {
    game: options.game,
    params: options.parameters,
    gpu: options.gpu,
    limits: options.limits,
    limitsPending: options.limitsPending,
    gpuPending: options.gpuPending,
    gpuUnavailable: options.gpuUnavailable,
    engineEnabled: options.engineEnabled,
    shippedIniKeys: options.shippedIniKeys,
  };
}

export function useEditorMutations(options: Options) {
  const queryClient = useQueryClient();
  const {
    game,
    configDir,
    runningExeName,
    overrideName,
    activeGameIdRef,
    setMessage,
    setApplyError,
    onApplied,
    onPresetApplied,
    onPresetApplyStart,
    onPresetApplyEnd,
    t,
  } = options;

  const applyCustomMutation = useMutation({
    mutationFn: async () => {
      assertApplyPlanAllowed(
        options.validationIssues,
        options.applyWarningsAcknowledged,
        t,
      );
      const snapshot = { gameId: game!.id, configDir };
      const { files, removals } = buildChanges(options);
      if (Object.keys(files).length === 0 && Object.keys(removals).length === 0) {
        throw new Error(t("errors.noChanges"));
      }
      const result = await applyCustom(
        snapshot.configDir,
        files,
        runningExeName ?? undefined,
        removals,
        snapshot.gameId,
        game?.engine_family,
        game?.engine_version,
        options.applyWarningsAcknowledged,
      );
      return { result, snapshot };
    },
    onMutate: () => setApplyError(undefined),
    onSuccess: ({ result, snapshot }) => {
      if (activeGameIdRef.current !== snapshot.gameId) return;
      onApplied();
      setMessage(
        t("applied", {
          count: result.diff.length,
          backupId: result.backup_id,
        }),
      );
      invalidateGameWorkspace(queryClient, snapshot.configDir, snapshot.gameId);
    },
    onError: (err) => setApplyError(formatInvokeError(err)),
  });

  const saveOverrideMutation = useMutation({
    mutationFn: async () => {
      assertPresetStorable(options.validationIssues, t);
      const snapshot = { gameId: game!.id, name: overrideName };
      const { files, removals } = buildChanges(options);
      await saveGameOverride({
        game_id: snapshot.gameId,
        name: snapshot.name,
        files,
        removals,
      });
      return snapshot;
    },
    onSuccess: (snapshot) => {
      if (activeGameIdRef.current !== snapshot.gameId) return;
      queryClient.invalidateQueries({ queryKey: ["overrides", snapshot.gameId] });
      setMessage(t("presetSaved", { name: snapshot.name }));
    },
    onError: (err) => setApplyError(formatInvokeError(err)),
  });

  const applyOverrideMutation = useMutation({
    mutationFn: async ({ override, warningsAcknowledged }: ApplyOverrideInput) => {
      const ctx = overrideValidationContext(options);
      if (ctx) {
        const issues = validateOverridePlan(override.files, override.removals, ctx);
        assertApplyPlanAllowed(issues, warningsAcknowledged, t);
      }
      const snapshot = { gameId: game!.id, configDir };
      const result = await applyGameOverride(
        snapshot.configDir,
        override,
        runningExeName ?? undefined,
        warningsAcknowledged,
      );
      return { result, snapshot };
    },
    onMutate: ({ override }) => {
      setApplyError(undefined);
      onPresetApplyStart?.(override.name);
    },
    onSuccess: ({ result, snapshot }) => {
      if (activeGameIdRef.current !== snapshot.gameId) return;
      onPresetApplied?.();
      setMessage(t("presetApplied", { backupId: result.backup_id }));
      invalidateGameWorkspace(queryClient, snapshot.configDir, snapshot.gameId);
    },
    onError: (err) => setApplyError(formatInvokeError(err)),
    onSettled: () => onPresetApplyEnd?.(),
  });

  const deleteOverrideMutation = useMutation({
    mutationFn: ({ gameId, name }: { gameId: string; name: string }) =>
      deleteGameOverride(gameId, name),
    onSuccess: (_result, variables) => {
      if (activeGameIdRef.current !== variables.gameId) return;
      queryClient.invalidateQueries({ queryKey: ["overrides", variables.gameId] });
    },
    onError: (err) => setApplyError(formatInvokeError(err)),
  });

  const importOverrideMutation = useMutation({
    mutationFn: (override: GameOverride) => {
      const ctx = overrideValidationContext(options);
      if (ctx) {
        const issues = validateOverridePlan(override.files, override.removals, ctx);
        assertPresetStorable(issues, t);
      }
      return saveGameOverride(override);
    },
    onSuccess: (_result, override) => {
      if (activeGameIdRef.current !== override.game_id) return;
      queryClient.invalidateQueries({ queryKey: ["overrides", override.game_id] });
      setMessage(t("presets.imported", { name: override.name }));
    },
    onError: (err) => setApplyError(formatInvokeError(err)),
  });

  return {
    applyCustomMutation,
    saveOverrideMutation,
    applyOverrideMutation,
    deleteOverrideMutation,
    importOverrideMutation,
  };
}
