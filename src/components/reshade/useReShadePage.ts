import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useMemo, useRef, useState } from "react";
import type { ReShadeDisclaimerKind } from "../ReShadeDisclaimerModal";
import {
  getReShadePresetDetails,
  getReShadeSettings,
  getReShadeStatus,
  getReShadeWorkspace,
  installReShade,
  launchGame,
  openGameFolder,
  removeReShade,
  setReShadeSettings,
  updateReShadePreset,
  updateReShadePresetParameters,
} from "../../lib/api";
import { formatInvokeError } from "../../lib/errors";
import { useDebouncedCallback } from "../../hooks/useDebouncedCallback";
import { useGameRunning } from "../../hooks/useGameRunning";
import { exeNameForRunningCheck } from "../../lib/gameRunning";
import {
  adaptedPresetId,
  apiLabel,
  buildPerGamePatch,
  effectivePreset,
  isGpuAdaptedPreset,
  isReShadeActiveForGame,
  mergeOverridePatch,
  presetIdForEditing,
  formatLaunchSuccess,
  resolvePrimaryAlert,
  savedGameApi,
  suggestApiForGame,
} from "../../lib/reshade";
import type {
  GameProfile,
  ReShadePresetOverrides,
  ReShadeSettings,
} from "../../lib/types";

interface PendingOverrideFlush {
  game: GameProfile;
  gameId: string;
  session: number;
  overrides: ReShadePresetOverrides;
  api: string;
  presetId: string;
  statusInstalled: boolean;
  settings: ReShadeSettings;
}

export function useReShadePage(game: GameProfile) {
  const queryClient = useQueryClient();
  const [disclaimer, setDisclaimer] = useState<ReShadeDisclaimerKind | null>(null);
  const [message, setMessage] = useState<string>();
  const [error, setError] = useState<string>();
  const [pendingOverrides, setPendingOverrides] = useState<ReShadePresetOverrides | null>(null);
  const overridesToFlushRef = useRef<PendingOverrideFlush | null>(null);
  const overridesGameIdRef = useRef(game.id);
  const activeGameIdRef = useRef(game.id);
  const mutationSessionRef = useRef(0);
  const selectedPresetRef = useRef<string | null>(null);
  activeGameIdRef.current = game.id;

  const invalidateForGame = (gameId: string) => {
    void queryClient.invalidateQueries({ queryKey: ["reshade-workspace", gameId] });
    void queryClient.invalidateQueries({ queryKey: ["reshade-settings", gameId] });
    void queryClient.invalidateQueries({ queryKey: ["reshade-status", gameId] });
  };

  const isActiveMutationSession = (session: number, gameId: string) =>
    session === mutationSessionRef.current && gameId === activeGameIdRef.current;

  const workspaceQuery = useQuery({
    queryKey: ["reshade-workspace", game.id],
    queryFn: () => getReShadeWorkspace(game),
    staleTime: 10_000,
  });

  const settingsFallbackQuery = useQuery({
    queryKey: ["reshade-settings", game.id],
    queryFn: () => getReShadeSettings(game.id, game.engine_family),
    enabled: workspaceQuery.isError,
    staleTime: 30_000,
  });

  const statusFallbackQuery = useQuery({
    queryKey: ["reshade-status", game.id],
    queryFn: () => getReShadeStatus(game),
    enabled: workspaceQuery.isError,
    staleTime: 10_000,
  });

  const workspaceError = workspaceQuery.isError
    ? formatInvokeError(workspaceQuery.error)
    : undefined;

  const settingsResponse = workspaceQuery.data?.settings ?? settingsFallbackQuery.data;
  const settings = settingsResponse?.settings;
  const presets = settingsResponse?.presets ?? [];
  const apis = settingsResponse?.apis ?? [];
  const status = workspaceQuery.data?.status ?? statusFallbackQuery.data;
  const bundleBinValid = status?.bundled_binaries_valid ?? false;
  const runningExeName = exeNameForRunningCheck(game.exe_name, status?.exe_path);
  const gameRunning = useGameRunning(runningExeName);

  const requestedPreset = useMemo(() => {
    if (!settings) return "clarity";
    return effectivePreset(settings, game.id);
  }, [settings, game.id]);

  const effectivePresetId = useMemo(
    () => adaptedPresetId(settings, game.id, status, settingsResponse),
    [settings, game.id, status, settingsResponse],
  );

  const installedPresetId = status?.active_preset ?? null;

  const gpuAdapted = isGpuAdaptedPreset(requestedPreset, effectivePresetId);

  const presetForEditing = useMemo(
    () => presetIdForEditing(settings, game.id, status, settingsResponse),
    [settings, game.id, status, settingsResponse],
  );

  const selectedApi = settings ? savedGameApi(settings, game.id) : null;

  const apiHint = suggestApiForGame(
    game,
    status?.suggested_api ?? workspaceQuery.data?.settings.suggested_api,
  );
  const effectiveApi = selectedApi ?? apiHint;
  const effectiveApiRef = useRef(effectiveApi);
  effectiveApiRef.current = effectiveApi;
  const presetForEditingRef = useRef(presetForEditing);
  presetForEditingRef.current = presetForEditing;
  const statusInstalledRef = useRef(false);
  statusInstalledRef.current = Boolean(status?.installed && effectiveApi);

  const effectiveOverrides: ReShadePresetOverrides = useMemo(() => {
    if (pendingOverrides) return pendingOverrides;
    return settings?.per_game[game.id]?.preset_overrides ?? {};
  }, [pendingOverrides, settings, game.id]);

  const presetDetailsQuery = useQuery({
    queryKey: ["reshade-preset-details", presetForEditing, game.id],
    queryFn: () => getReShadePresetDetails(presetForEditing, game.id),
    enabled: !!presetForEditing && isReShadeActiveForGame(settings, game.id),
  });

  const saveSettingsMutation = useMutation({
    mutationFn: ({ settings }: { gameId: string; session: number; settings: ReShadeSettings }) =>
      setReShadeSettings(settings),
    onSuccess: (_result, variables) => {
      if (!isActiveMutationSession(variables.session, variables.gameId)) return;
      setError(undefined);
      invalidateForGame(variables.gameId);
    },
    onError: (err, variables) => {
      if (!isActiveMutationSession(variables.session, variables.gameId)) return;
      setError(formatInvokeError(err));
    },
  });

  const installMutation = useMutation({
    mutationFn: (vars: {
      gameId: string;
      session: number;
      api: string;
      preset: string;
    }) => installReShade(game, vars.api, vars.preset),
    onSuccess: (result, variables) => {
      if (!isActiveMutationSession(variables.session, variables.gameId)) return;
      setDisclaimer(null);
      const warn = result.warnings?.length ? ` ${result.warnings.join(" ")}` : "";
      setMessage(
        `ReShade установлен (${apiLabel(result.graphics_api)}).${warn} Запустите игру — Home для меню ReShade.`,
      );
      setError(undefined);
      invalidateForGame(variables.gameId);
    },
    onError: (err, variables) => {
      if (!isActiveMutationSession(variables.session, variables.gameId)) return;
      setDisclaimer(null);
      setError(formatInvokeError(err));
    },
  });

  const removeMutation = useMutation({
    mutationFn: (_vars: { gameId: string; session: number }) => removeReShade(game),
    onSuccess: (result, variables) => {
      if (!isActiveMutationSession(variables.session, variables.gameId)) return;
      const base = status?.broken_install
        ? "Повреждённая установка очищена — можно запускать игру."
        : "ReShade удалён из папки игры.";
      const warn = result.warnings?.length ? ` ${result.warnings.join(" ")}` : "";
      setMessage(`${base}${warn}`);
      setError(undefined);
      invalidateForGame(variables.gameId);
    },
    onError: (err, variables) => {
      if (!isActiveMutationSession(variables.session, variables.gameId)) return;
      setError(formatInvokeError(err));
    },
  });

  const updatePresetMutation = useMutation({
    mutationFn: (vars: {
      gameId: string;
      session: number;
      api: string;
      presetId: string;
    }) => updateReShadePreset(game, vars.api, vars.presetId),
    onSuccess: (result, variables) => {
      if (!isActiveMutationSession(variables.session, variables.gameId)) return;
      const warn = result.warnings?.length ? ` ${result.warnings[0]}` : "";
      setMessage(`Пресет «${result.preset_id}» применён.${warn}`);
      setError(undefined);
      invalidateForGame(variables.gameId);
    },
    onError: (err, variables) => {
      if (!isActiveMutationSession(variables.session, variables.gameId)) return;
      setError(formatInvokeError(err));
    },
  });

  const launchSkipMutation = useMutation({
    mutationFn: (_vars: { gameId: string; session: number }) => launchGame(game, true),
    onSuccess: (result, variables) => {
      if (!isActiveMutationSession(variables.session, variables.gameId)) return;
      setMessage(formatLaunchSuccess(result));
      invalidateForGame(variables.gameId);
      void queryClient.invalidateQueries({ queryKey: ["game-running"] });
    },
    onError: (err, variables) => {
      if (!isActiveMutationSession(variables.session, variables.gameId)) return;
      setError(formatInvokeError(err));
    },
  });

  const updateOverridesMutation = useMutation({
    mutationFn: (vars: {
      game: GameProfile;
      overrides: ReShadePresetOverrides;
      api: string;
      gameId: string;
      session: number;
      presetId: string;
    }) =>
      updateReShadePresetParameters(vars.game, vars.api, vars.presetId, vars.overrides),
    onSuccess: (_result, variables) => {
      if (!isActiveMutationSession(variables.session, variables.gameId)) return;
      setPendingOverrides(null);
      overridesToFlushRef.current = null;
      setError(undefined);
      invalidateForGame(variables.gameId);
    },
    onError: (err, variables) => {
      if (!isActiveMutationSession(variables.session, variables.gameId)) return;
      setError(formatInvokeError(err));
    },
  });

  const [patchOverridesDebounced, flushOverridesNow] = useDebouncedCallback(() => {
    // Снимок контекста (game/api/preset/settings/session) делается в момент правки —
    // поэтому отложенная запись всегда уходит в нужную игру, даже если пользователь
    // успел переключиться до срабатывания debounce.
    const snap = overridesToFlushRef.current;
    if (!snap) return;
    overridesToFlushRef.current = null;

    if (snap.statusInstalled && snap.api) {
      updateOverridesMutation.mutate({
        game: snap.game,
        overrides: snap.overrides,
        api: snap.api,
        gameId: snap.gameId,
        session: snap.session,
        presetId: snap.presetId,
      });
      return;
    }

    saveSettingsMutation.mutate({
      gameId: snap.gameId,
      session: snap.session,
      settings: buildPerGamePatch(snap.settings, snap.gameId, {
        preset_overrides: snap.overrides,
      }),
    });
  }, 400);

  useEffect(() => {
    mutationSessionRef.current += 1;
    activeGameIdRef.current = game.id;
    setPendingOverrides(null);
    overridesToFlushRef.current = null;
    setMessage(undefined);
    setError(undefined);
    setDisclaimer(null);
    return () => {
      flushOverridesNow();
    };
  }, [game.id, flushOverridesNow]);

  const launchWithoutReShade = () => {
    flushOverridesNow();
    launchSkipMutation.mutate({ gameId: game.id, session: mutationSessionRef.current });
  };

  const removeCurrentGameReShade = () => {
    removeMutation.mutate({ gameId: game.id, session: mutationSessionRef.current });
  };

  const patchSettings = (patch: Partial<ReShadeSettings>) => {
    if (!settings) return;
    saveSettingsMutation.mutate({
      gameId: game.id,
      session: mutationSessionRef.current,
      settings: { ...settings, ...patch },
    });
  };

  const selectApi = (api: string) => {
    if (!settings) return;
    saveSettingsMutation.mutate({
      gameId: game.id,
      session: mutationSessionRef.current,
      settings: buildPerGamePatch(settings, game.id, {
        api,
        api_remembered: true,
        enabled: settings.per_game[game.id]?.enabled ?? true,
        preset: requestedPreset,
      }),
    });
  };

  const handleGlobalToggle = (enabled: boolean) => {
    if (!settings) return;
    if (enabled && !settings.warnings_acknowledged) {
      setDisclaimer("enable");
      return;
    }
    patchSettings({ global_enabled: enabled });
  };

  const confirmDisclaimer = () => {
    if (!settings || !disclaimer) return;

    if (disclaimer === "enable") {
      const session = mutationSessionRef.current;
      saveSettingsMutation.mutate(
        {
          gameId: game.id,
          session,
          settings: { ...settings, global_enabled: true, warnings_acknowledged: true },
        },
        {
          onSuccess: () => {
            if (!isActiveMutationSession(session, game.id)) return;
            setDisclaimer(null);
            setMessage("ReShade включён для всех игр.");
          },
        },
      );
      return;
    }

    if (disclaimer === "install") {
      if (!bundleBinValid) {
        setDisclaimer(null);
        setError("ReShade не встроен в сборку — см. технические детали.");
        return;
      }
      const proceed = () => {
        const api = selectedApi ?? apiHint;
        const runInstall = () =>
          installMutation.mutate({
            gameId: game.id,
            session: mutationSessionRef.current,
            api,
            preset: requestedPreset,
          });
        if (!selectedApi && settings) {
          const session = mutationSessionRef.current;
          saveSettingsMutation.mutate(
            {
              gameId: game.id,
              session,
              settings: buildPerGamePatch(settings, game.id, {
                api,
                api_remembered: true,
                enabled: settings.per_game[game.id]?.enabled ?? true,
                preset: requestedPreset,
              }),
            },
            { onSuccess: () => isActiveMutationSession(session, game.id) && runInstall() },
          );
          return;
        }
        runInstall();
      };
      if (!settings.install_warning_acknowledged) {
        const session = mutationSessionRef.current;
        saveSettingsMutation.mutate(
          {
            gameId: game.id,
            session,
            settings: { ...settings, install_warning_acknowledged: true },
          },
          { onSuccess: () => isActiveMutationSession(session, game.id) && proceed() },
        );
      } else {
        proceed();
      }
    }
  };

  const handlePerGameToggle = (enabled: boolean) => {
    if (!settings) return;
    saveSettingsMutation.mutate({
      gameId: game.id,
      session: mutationSessionRef.current,
      settings: buildPerGamePatch(settings, game.id, { enabled }),
    });
  };

  const handlePresetSelect = (presetId: string) => {
    if (!settings) return;
    flushOverridesNow();
    overridesToFlushRef.current = null;
    setPendingOverrides(null);
    const api = effectiveApiRef.current;
    const session = mutationSessionRef.current;
    selectedPresetRef.current = presetId;
    saveSettingsMutation.mutate(
      {
        gameId: game.id,
        session,
        settings: buildPerGamePatch(settings, game.id, { preset: presetId }),
      },
      {
        onSuccess: () => {
          if (!isActiveMutationSession(session, game.id)) return;
          if (selectedPresetRef.current !== presetId) return;
          if (status?.installed && api) {
            updatePresetMutation.mutate({
              gameId: game.id,
              session,
              api,
              presetId,
            });
          }
        },
      },
    );
  };

  const patchOverrides = (patch: Partial<ReShadePresetOverrides>) => {
    if (!settings) return;
    overridesGameIdRef.current = game.id;
    setPendingOverrides((prev) => {
      const base = prev ?? settings.per_game[game.id]?.preset_overrides ?? {};
      const next = mergeOverridePatch(base, patch);
      overridesToFlushRef.current = {
        game,
        gameId: game.id,
        session: mutationSessionRef.current,
        overrides: next,
        api: effectiveApi,
        presetId: presetForEditing,
        statusInstalled: Boolean(status?.installed && effectiveApi),
        settings,
      };
      return next;
    });
    patchOverridesDebounced();
  };

  const globallyOn = settings?.global_enabled ?? false;
  const activeForGame = isReShadeActiveForGame(settings, game.id);
  const gameBlockDisabled = !globallyOn;
  const presetDetails = presetDetailsQuery.data;
  const gpuAdaptReason =
    status?.gpu_adapt_reason ?? workspaceQuery.data?.settings.gpu_adapt_reason;
  const gpuName = status?.gpu_name ?? workspaceQuery.data?.settings.gpu_name;

  const pageAlert = resolvePrimaryAlert({
    error: workspaceError ?? error,
    brokenInstall: !!status?.broken_install,
    bundleBinValid,
    showDisclaimer: false,
  });

  const requestedPresetName =
    presets.find((p) => p.id === requestedPreset)?.name ?? requestedPreset;
  const installedPresetName = installedPresetId
    ? (presets.find((p) => p.id === installedPresetId)?.name ?? installedPresetId)
    : gpuAdapted
      ? (presets.find((p) => p.id === effectivePresetId)?.name ?? effectivePresetId)
      : null;

  const primaryCtaLabel = status?.installed ? "Обновить ReShade" : "Установить ReShade";
  const primaryCtaLoading = installMutation.isPending || updatePresetMutation.isPending;
  const canPrimaryCta =
    !gameRunning &&
    !gameBlockDisabled &&
    activeForGame &&
    bundleBinValid &&
    !status?.broken_install &&
    !!effectiveApi;
  const canRemoveReShade =
    !gameRunning && (!!status?.installed || !!status?.broken_install);
  const canLaunchWithoutReShade = !gameRunning;

  const requestInstall = () => {
    if (!effectiveApi) {
      setError("Сначала выберите графический API.");
      return;
    }
    if (!selectedApi && settings) {
      selectApi(effectiveApi);
    }
    setDisclaimer("install");
  };

  const runPrimaryCta = () => {
    const api = effectiveApi;
    if (status?.installed) {
      if (!selectedApi && settings) selectApi(api);
      updatePresetMutation.mutate({
        gameId: game.id,
        session: mutationSessionRef.current,
        api,
        presetId: requestedPreset,
      });
      return;
    }
    requestInstall();
  };

  return {
    game,
    settings,
    presets,
    apis,
    status,
    bundleBinValid,
    requestedPreset,
    effectivePresetId,
    installedPresetId,
    gpuAdapted,
    requestedPresetName,
    installedPresetName,
    presetForEditing,
    selectedApi,
    presetDetails,
    effectiveOverrides,
    apiHint,
    effectiveApi,
    gpuAdaptReason,
    gpuName,
    pageAlert,
    globallyOn,
    activeForGame,
    gameBlockDisabled,
    message,
    disclaimer,
    primaryCtaLabel,
    primaryCtaLoading,
    canPrimaryCta,
    canRemoveReShade,
    canLaunchWithoutReShade,
    gameRunning,
    runningExeName,
    workspaceQuery,
    workspaceError,
    settingsQuery: workspaceQuery,
    settingsFallbackQuery,
    saveSettingsMutation,
    installMutation,
    removeMutation,
    updatePresetMutation,
    launchSkipMutation,
    launchWithoutReShade,
    removeCurrentGameReShade,
    updateOverridesMutation,
    setDisclaimer,
    patchSettings,
    selectApi,
    handleGlobalToggle,
    confirmDisclaimer,
    handlePerGameToggle,
    handlePresetSelect,
    patchOverrides,
    runPrimaryCta,
    openGameFolder: () => {
      void openGameFolder(game).catch((err) => setError(formatInvokeError(err)));
    },
  };
}

export type ReShadePageState = ReturnType<typeof useReShadePage>;
