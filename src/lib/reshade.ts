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
  const base = `Запуск через ${result.launcher}…`;
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

/** Пресет после GPU-адаптации (что реально ставится в игру). */
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

/** Пресет для UI слайдеров: установленный в игре или адаптированный к установке. */
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

export const BLOCKED_BROKEN_RESHADE_LAUNCH_MSG =
  "Повреждённая установка ReShade и в приложении нет рабочих DLL. " +
  "Откройте вкладку ReShade → «Удалить» или нажмите «Без ReShade».";

/** Блокирует запуск с ReShade (не «Без ReShade»), если proxy сломан и нет DLL в бандле. */
export function blocksReShadeLaunch(status: ReShadeGameStatus | undefined): string | null {
  if (status?.broken_install && !status.bundled_binaries_valid) {
    return BLOCKED_BROKEN_RESHADE_LAUNCH_MSG;
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

/** Значения по умолчанию при включении эффекта, которого нет в базовом пресете. */
export const ReShadeEffectDefaultParams: Record<string, Record<string, string>> = {
  Vignette: {
    Amount: "-0.350000",
    Slope: "8",
    Radius: "1.000000",
    Ratio: "0.000000",
  },
};

export const ReShadeSliderParams = [
  { effect: "AdaptiveSharpen", key: "sharp_strength", label: "Резкость", min: 0, max: 2, step: 0.05 },
  { effect: "Clarity", key: "ClarityRadius", label: "Радиус чёткости", min: 0, max: 10, step: 0.25 },
  { effect: "Clarity", key: "ClarityOffset", label: "Смещение чёткости", min: 0, max: 5, step: 0.1 },
  { effect: "Clarity", key: "ClarityMaskIntensity", label: "Маска чёткости", min: 0, max: 1, step: 0.05 },
  { effect: "Vignette", key: "Amount", label: "Сила виньетки", min: -1, max: 0, step: 0.05 },
  { effect: "Vignette", key: "Radius", label: "Радиус виньетки", min: 0, max: 2, step: 0.05 },
  { effect: "Vibrance", key: "Vibrance", label: "Насыщенность", min: -1, max: 1, step: 0.05 },
  { effect: "LiftGammaGain", key: "RGB_Gain", label: "Яркость (gain)", min: 0.5, max: 1.5, step: 0.02 },
] as const;

export const ReShadeEffectLabels: Record<string, { label: string; hint: string }> = {
  Clarity: {
    label: "Чёткость",
    hint: "Локальный контраст — детали без общей резкости",
  },
  Vignette: {
    label: "Виньетка",
    hint: "Затемнение по краям кадра",
  },
  AdaptiveSharpen: {
    label: "Адаптивная резкость",
    hint: "Чёткие края без цветных ореолов",
  },
  Vibrance: {
    label: "Насыщенность",
    hint: "Мягко усиливает цвета",
  },
  LiftGammaGain: {
    label: "Яркость и контраст",
    hint: "Тени, средние тона и свет",
  },
};

export function reshadeEffectLabel(effectId: string): string {
  return ReShadeEffectLabels[effectId]?.label ?? effectId;
}

export function reshadeEffectHint(effectId: string): string | undefined {
  return ReShadeEffectLabels[effectId]?.hint;
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
      return "Не выбран";
  }
}

export function engineApiHint(game: GameProfile): string {
  if (game.engine_family === "ue5") return "UE5 → обычно DirectX 12 (dxgi.dll)";
  if (game.engine_family === "ue4") return "UE4 → обычно DirectX 11 (d3d11.dll)";
  if (game.engine_family === "forza") return "Forza → обычно DirectX 12 (dxgi.dll)";
  if (game.is_unity) return "Unity → обычно DirectX 11";
  return "Выберите API, который использует игра";
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
    return { tone: "muted", label: "ReShade выключен глобально" };
  }
  if (!activeForGame) {
    return { tone: "muted", label: "ReShade выключен для этой игры" };
  }
  if (status?.broken_install) {
    return {
      tone: "danger",
      label: "Повреждённая установка",
      detail:
        "Некорректный proxy в папке игры — «Удалить» на вкладке ReShade или «Играть» в шапке для авто-восстановления",
    };
  }
  if (status?.installed) {
    return { tone: "success", label: "Готов к запуску" };
  }
  return { tone: "warning", label: "Не установлен" };
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
      title: "Ошибка",
      message: input.error,
    };
  }
  if (input.brokenInstall) {
    return {
      kind: "broken",
      tone: "error",
      title: "Повреждённая установка",
      message:
        "В папке игры остался некорректный ReShade (битый proxy или неверный файл). " +
        "Нажмите «Удалить» ниже или запустите игру кнопкой «Играть» в шапке — GSM попробует восстановить установку.",
      actionLabel: "Удалить",
    };
  }
  if (!input.bundleBinValid) {
    return {
      kind: "no_dll",
      tone: "warning",
      title: "ReShade не готов к установке",
      message:
        "ReShade DLL не найдены в установке приложения. Переустановите Game Settings Master или нажмите «Без ReShade».",
    };
  }
  if (input.showDisclaimer) {
    return {
      kind: "disclaimer",
      tone: "warning",
      title: "Внимание",
      message:
        "GSM не проверяет античит. ReShade может нарушать правила онлайн-игр. Используйте на свой риск.",
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
    return "● ReShade выключен глобально · proxy удаляется при запуске";
  }
  if (!input.activeForGame) {
    return "● Выключен для этой игры · proxy удаляется при запуске";
  }
  if (input.brokenInstall) return "● Ошибка в папке игры";
  const api = input.selectedApi ? apiLabel(input.selectedApi) : "API не выбран";
  if (input.installed) {
    const preset =
      input.installedPresetName && input.gpuAdapted
        ? `${input.installedPresetName} (выбран ${input.requestedPresetName})`
        : (input.installedPresetName ?? input.requestedPresetName);
    return `● Установлен · ${api} · ${preset}`;
  }
  if (input.gpuAdapted && input.installedPresetName) {
    return `● Не установлен · ${api} · будет ${input.installedPresetName}`;
  }
  return `● Не установлен · ${api} · ${input.requestedPresetName}`;
}

export function presetAccentForReShade(id: string): string {
  if (ReShadePresetAccents[id]) return ReShadePresetAccents[id];
  let hash = 0;
  for (let i = 0; i < id.length; i++) {
    hash = (hash * 31 + id.charCodeAt(i)) >>> 0;
  }
  return `hsl(${hash % 360} 45% 55%)`;
}
