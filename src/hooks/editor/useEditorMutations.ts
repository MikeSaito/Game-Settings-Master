import { useMutation, useQueryClient } from "@tanstack/react-query";
import type { MutableRefObject } from "react";
import {
  applyCustom,
  applyGameOverride,
  deleteGameOverride,
  saveGameOverride,
} from "@/lib/api";
import { buildCustomChanges } from "@/lib/editor";
import { filterParamsByPanel, type EditorPanel } from "@/lib/routing";
import { formatInvokeError } from "@/lib/core";
import type { GameOverride, GameParameter, GameProfile, GpuCapabilities } from "@/lib/core";

type Overrides = GameOverride[];

interface Options {
  game: GameProfile | null;
  configDir: string;
  runningExeName: string | null;
  params: GameParameter[];
  parameters: GameParameter[];
  panel: EditorPanel;
  gpu: GpuCapabilities | undefined;
  engineEnabled: Set<string>;
  editableCategories: Set<string>;
  overrideName: string;
  activeGameIdRef: MutableRefObject<string | undefined>;
  setMessage: (message: string | undefined) => void;
  setApplyError: (error: string | undefined) => void;
  onApplied: () => void;
  t: (key: string, options?: Record<string, unknown>) => string;
}

function buildChanges(options: Options) {
  return buildCustomChanges(
    filterParamsByPanel(options.params, options.panel),
    options.parameters,
    options.gpu,
    options.engineEnabled,
    options.editableCategories,
  );
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
    t,
  } = options;

  const applyCustomMutation = useMutation({
    mutationFn: async () => {
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
      queryClient.invalidateQueries({
        queryKey: ["backups", snapshot.configDir, snapshot.gameId],
      });
      queryClient.invalidateQueries({
        queryKey: ["parameters", snapshot.configDir, snapshot.gameId],
      });
      queryClient.invalidateQueries({ queryKey: ["game-config"] });
    },
    onError: (err) => setApplyError(formatInvokeError(err)),
  });

  const saveOverrideMutation = useMutation({
    mutationFn: async () => {
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
    mutationFn: async (override: Overrides[number]) => {
      const snapshot = { gameId: game!.id, configDir };
      const result = await applyGameOverride(
        snapshot.configDir,
        override,
        runningExeName ?? undefined,
      );
      return { result, snapshot };
    },
    onSuccess: ({ result, snapshot }) => {
      if (activeGameIdRef.current !== snapshot.gameId) return;
      setMessage(t("presetApplied", { backupId: result.backup_id }));
      queryClient.invalidateQueries({
        queryKey: ["backups", snapshot.configDir, snapshot.gameId],
      });
      queryClient.invalidateQueries({
        queryKey: ["parameters", snapshot.configDir, snapshot.gameId],
      });
      queryClient.invalidateQueries({ queryKey: ["game-config"] });
    },
    onError: (err) => setApplyError(formatInvokeError(err)),
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
    mutationFn: (override: GameOverride) => saveGameOverride(override),
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
