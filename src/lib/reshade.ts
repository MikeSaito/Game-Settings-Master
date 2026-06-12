import i18n from "../i18n";
import type {
  GameProfile,
  LaunchResult,
  ReShadeGameStatus,
  ReShadePresetOverrides,
  ReShadeSettings,
} from "./types";

const KNOWN_RESHADE_APIS = new Set(["dx9", "dx11", "dx12", "opengl", "vulkan"]);

export function isValidReShadeApi(api: string): boolean {
  return KNOWN_RESHADE_APIS.has(api);
}

export function formatLaunchSuccess(result: LaunchResult): string {
  const base = i18n.t("reshade:lib.launchVia", { launcher: result.launcher });
  return result.warning ? `${base} ${result.warning}` : base;
}

export function isReShadeActiveForGame(
  settings: ReShadeSettings | undefined,
  gameId: string,
): boolean {
  if (!settings?.global_enabled) return false;
  return settings.per_game[gameId]?.enabled ?? true;
}

export function effectivePreset(settings: ReShadeSettings, gameId: string): string {
  return settings.per_game[gameId]?.preset ?? settings.default_preset;
}

/** Preset after GPU adaptation (what is actually installed in the game). */
export function adaptedPresetId(
  settings: ReShadeSettings | undefined,
  gameId: string,
  status?: ReShadeGameStatus | null,
  settingsResponse?: { effective_preset?: string | null } | null,
): string {
  if (status?.effective_preset) return status.effective_preset;
  if (settingsResponse?.effective_preset) return settingsResponse.effective_preset;
  if (!settings) return "clarity";
  return effectivePreset(settings, gameId);
}

/** Preset for UI sliders: installed in the game or adapted for install. */
export function presetIdForEditing(
  settings: ReShadeSettings | undefined,
  gameId: string,
  status?: ReShadeGameStatus | null,
  settingsResponse?: { effective_preset?: string | null } | null,
): string {
  if (status?.installed && status.active_preset) return status.active_preset;
  return adaptedPresetId(settings, gameId, status, settingsResponse);
}

export function isGpuAdaptedPreset(
  requested: string,
  effective: string,
): boolean {
  return requested !== effective;
}

export function savedGameApi(settings: ReShadeSettings, gameId: string): string | null {
  return settings.per_game[gameId]?.api ?? null;
}

export function shouldPromptApi(settings: ReShadeSettings, gameId: string): boolean {
  const game = settings.per_game[gameId];
  if (!game) return true;
  if (!game.api_remembered || !game.api) return true;
  return !isValidReShadeApi(game.api);
}

export function buildPerGamePatch(
  settings: ReShadeSettings,
  gameId: string,
  patch: Partial<import("./types").ReShadePerGameSettings>,
): ReShadeSettings {
  const current = settings.per_game[gameId] ?? {
    enabled: true,
    api: null,
    api_remembered: false,
    preset: null,
    preset_overrides: null,
  };
  return {
    ...settings,
    per_game: {
      ...settings.per_game,
      [gameId]: { ...current, ...patch },
    },
  };
}

export function mergeOverridePatch(
  current: ReShadePresetOverrides,
  patch: Partial<ReShadePresetOverrides>,
): ReShadePresetOverrides {
  const next: ReShadePresetOverrides = { ...current, ...patch };
  if (patch.parameters) {
    next.parameters = { ...current.parameters };
    for (const [effect, keys] of Object.entries(patch.parameters)) {
      next.parameters[effect] = { ...current.parameters?.[effect], ...keys };
    }
  }
  if (patch.behavior) {
    next.behavior = { ...current.behavior, ...patch.behavior };
  }
  return next;
}

export function blockedBrokenReShadeLaunchMsg(): string {
  return i18n.t("reshade:lib.blockedBrokenLaunch");
}

/** Blocks launch with ReShade when proxy is broken and no DLL in bundle. */
export function blocksReShadeLaunch(status: ReShadeGameStatus | undefined): string | null {
  if (status?.broken_install && !status.bundled_binaries_valid) {
    return blockedBrokenReShadeLaunchMsg();
  }
  return null;
}

export function suggestApiForGame(
  game: GameProfile,
  knownSuggested?: string | null,
): string {
  if (knownSuggested) return knownSuggested;
  if (game.engine_family === "ue5") return "dx12";
  if (game.engine_family === "ue4") return "dx11";
  if (game.is_unity) return "dx11";
  return "dx12";
}

export const ReShadeFineTuneEffects = ["Clarity", "Vignette", "AdaptiveSharpen"] as const;

/** Default values when enabling an effect not in the base preset. */
export const ReShadeEffectDefaultParams: Record<string, Record<string, string>> = {
  Vignette: {
    Amount: "-0.350000",
    Slope: "8",
    Radius: "1.000000",
    Ratio: "0.000000",
  },
};

export const ReShadeSliderParams = [
  { effect: "AdaptiveSharpen", key: "sharp_strength", min: 0, max: 2, step: 0.05 },
  { effect: "Clarity", key: "ClarityRadius", min: 0, max: 10, step: 0.25 },
  { effect: "Clarity", key: "ClarityOffset", min: 0, max: 5, step: 0.1 },
  { effect: "Clarity", key: "ClarityMaskIntensity", min: 0, max: 1, step: 0.05 },
  { effect: "Vignette", key: "Amount", min: -1, max: 0, step: 0.05 },
  { effect: "Vignette", key: "Radius", min: 0, max: 2, step: 0.05 },
  { effect: "Vibrance", key: "Vibrance", min: -1, max: 1, step: 0.05 },
  { effect: "LiftGammaGain", key: "RGB_Gain", min: 0.5, max: 1.5, step: 0.02 },
] as const;

export function reshadeSliderLabel(effect: string, key: string): string {
  return i18n.t(`reshade:lib.sliders.${effect}_${key}`, { defaultValue: key });
}

export function reshadeEffectLabel(effectId: string): string {
  return i18n.t(`reshade:lib.effects.${effectId}.label`, { defaultValue: effectId });
}

export function reshadeEffectHint(effectId: string): string | undefined {
  const hint = i18n.t(`reshade:lib.effects.${effectId}.hint`, { defaultValue: "" });
  return hint || undefined;
}

export function apiLabel(apiId: string | null | undefined): string {
  switch (apiId) {
    case "dx9":
      return "DirectX 9";
    case "dx11":
      return "DirectX 10/11";
    case "dx12":
      return "DirectX 12";
    case "opengl":
      return "OpenGL";
    case "vulkan":
      return "Vulkan";
    default:
      return i18n.t("reshade:lib.apiNotSelected");
  }
}

export function engineApiHint(game: GameProfile): string {
  if (game.engine_family === "ue5") return i18n.t("reshade:lib.engineApiUe5");
  if (game.engine_family === "ue4") return i18n.t("reshade:lib.engineApiUe4");
  if (game.engine_family === "forza") return i18n.t("reshade:lib.engineApiForza");
  if (game.is_unity) return i18n.t("reshade:lib.engineApiUnity");
  return i18n.t("reshade:lib.engineApiGeneric");
}

export type ReShadeHealthTone = "success" | "warning" | "danger" | "muted";

export interface ReShadeHealthSummary {
  tone: ReShadeHealthTone;
  label: string;
  detail?: string;
}

export function deriveHealthSummary(
  status: ReShadeGameStatus | undefined,
  globallyOn: boolean,
  activeForGame: boolean,
): ReShadeHealthSummary {
  if (!globallyOn) {
    return { tone: "muted", label: i18n.t("reshade:lib.healthGlobalOff") };
  }
  if (!activeForGame) {
    return { tone: "muted", label: i18n.t("reshade:lib.healthGameOff") };
  }
  if (status?.broken_install) {
    return {
      tone: "danger",
      label: i18n.t("reshade:lib.healthBroken"),
      detail: i18n.t("reshade:lib.healthBrokenDetail"),
    };
  }
  if (status?.installed) {
    return { tone: "success", label: i18n.t("reshade:lib.healthReady") };
  }
  return { tone: "warning", label: i18n.t("reshade:lib.healthNotInstalled") };
}

export type ReShadeAlertKind = "error" | "broken" | "no_dll" | "disclaimer";

export interface ReShadeAlertConfig {
  kind: ReShadeAlertKind;
  tone: "error" | "warning";
  title: string;
  message: string;
  actionLabel?: string;
}

export function resolvePrimaryAlert(input: {
  error?: string;
  brokenInstall: boolean;
  bundleBinValid: boolean;
  showDisclaimer: boolean;
}): ReShadeAlertConfig | null {
  if (input.error) {
    return {
      kind: "error",
      tone: "error",
      title: i18n.t("reshade:lib.alertError"),
      message: input.error,
    };
  }
  if (input.brokenInstall) {
    return {
      kind: "broken",
      tone: "error",
      title: i18n.t("reshade:lib.alertBrokenTitle"),
      message: i18n.t("reshade:lib.alertBrokenMessage"),
      actionLabel: i18n.t("reshade:lib.alertRemove"),
    };
  }
  if (!input.bundleBinValid) {
    return {
      kind: "no_dll",
      tone: "warning",
      title: i18n.t("reshade:lib.alertNoDllTitle"),
      message: i18n.t("reshade:lib.alertNoDllMessage"),
    };
  }
  if (input.showDisclaimer) {
    return {
      kind: "disclaimer",
      tone: "warning",
      title: i18n.t("reshade:lib.alertDisclaimerTitle"),
      message: i18n.t("reshade:lib.alertDisclaimerMessage"),
    };
  }
  return null;
}

export const ReShadePresetAccents: Record<string, string> = {
  performance: "#7bc96f",
  clarity: "#5b8def",
  cinematic: "#9b6bdf",
};

export function apiProxyFile(apiId: string | null | undefined): string {
  switch (apiId) {
    case "dx9":
      return "d3d9.dll";
    case "dx11":
      return "d3d11.dll";
    case "dx12":
      return "dxgi.dll";
    case "opengl":
      return "opengl32.dll";
    case "vulkan":
      return "ReShade64.dll";
    default:
      return "";
  }
}

export function formatReShadeStatusMeta(input: {
  globallyOn: boolean;
  activeForGame: boolean;
  installed: boolean;
  brokenInstall: boolean;
  selectedApi: string | null;
  requestedPresetName: string;
  installedPresetName?: string | null;
  gpuAdapted?: boolean;
}): string {
  if (!input.globallyOn) {
    return i18n.t("reshade:lib.statusGlobalOff");
  }
  if (!input.activeForGame) {
    return i18n.t("reshade:lib.statusGameOff");
  }
  if (input.brokenInstall) return i18n.t("reshade:lib.statusBroken");
  const api = input.selectedApi ? apiLabel(input.selectedApi) : i18n.t("reshade:lib.statusApiNotSelected");
  if (input.installed) {
    const preset =
      input.installedPresetName && input.gpuAdapted
        ? i18n.t("reshade:lib.statusPresetAdapted", {
            installed: input.installedPresetName,
            requested: input.requestedPresetName,
          })
        : (input.installedPresetName ?? input.requestedPresetName);
    return i18n.t("reshade:lib.statusInstalled", { api, preset });
  }
  if (input.gpuAdapted && input.installedPresetName) {
    return i18n.t("reshade:lib.statusWillInstall", {
      api,
      preset: input.installedPresetName,
    });
  }
  return i18n.t("reshade:lib.statusNotInstalled", {
    api,
    preset: input.requestedPresetName,
  });
}

export function presetAccentForReShade(id: string): string {
  if (ReShadePresetAccents[id]) return ReShadePresetAccents[id];
  let hash = 0;
  for (let i = 0; i < id.length; i++) {
    hash = (hash * 31 + id.charCodeAt(i)) >>> 0;
  }
  return `hsl(${hash % 360} 45% 55%)`;
}
