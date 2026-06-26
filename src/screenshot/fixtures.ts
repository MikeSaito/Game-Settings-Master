import type { BackupInfo, GameParameter, GameProfile, GpuCapabilities } from "@/lib/core";
import type { AppLanguage } from "@/i18n";

export const screenshotGpu: GpuCapabilities = {
  vendor: "nvidia",
  name: "NVIDIA GeForce RTX 4070",
  supports_dlss: true,
  supports_dlss_fg: true,
  supports_ray_tracing: true,
};

const screenshotGameDefs: Omit<GameProfile, "name">[] = [
  {
    id: "game-1",
    source: "steam",
    install_dir: "C:\\Games\\Game1",
    config_dir: "C:\\Games\\Game1\\Saved\\Config\\Windows",
    exe_name: "Game1-Win64-Shipping.exe",
    is_ue: true,
    possible_ue: true,
    cover_url: null,
    custom_cover: null,
    build_id: "12345678",
    engine_family: "ue5",
    engine_version: "5.3",
  },
  {
    id: "game-2",
    source: "steam",
    install_dir: "C:\\Games\\Game2",
    config_dir: "C:\\Games\\Game2\\Saved\\Config\\Windows",
    exe_name: "Game2-Win64-Shipping.exe",
    is_ue: true,
    possible_ue: true,
    cover_url: null,
    custom_cover: null,
    build_id: null,
    engine_family: "ue5",
    engine_version: "5.0",
  },
  {
    id: "game-3",
    source: "epic",
    install_dir: "C:\\Games\\Game3",
    config_dir: "C:\\Games\\Game3\\Saved\\Config\\WindowsNoEditor",
    exe_name: "Game3.exe",
    is_ue: true,
    possible_ue: true,
    cover_url: null,
    custom_cover: null,
    build_id: null,
    engine_family: "ue4",
    engine_version: "4.27",
  },
  {
    id: "game-4",
    source: "manual",
    install_dir: "D:\\Dev\\Game4",
    config_dir: "D:\\Dev\\Game4\\Saved\\Config\\Windows",
    exe_name: "Game4-Win64-Shipping.exe",
    is_ue: true,
    possible_ue: true,
    cover_url: null,
    custom_cover: null,
    build_id: null,
    engine_family: "ue5",
    engine_version: "5.4",
  },
];

const gameNames: Record<AppLanguage, string[]> = {
  ru: ["Игра 1", "Игра 2", "Игра 3", "Игра 4"],
  en: ["Game 1", "Game 2", "Game 3", "Game 4"],
};

export function getScreenshotGames(lang: AppLanguage): GameProfile[] {
  return screenshotGameDefs.map((game, index) => ({
    ...game,
    name: gameNames[lang][index] ?? `Game ${index + 1}`,
  }));
}

export function getScreenshotGame(lang: AppLanguage): GameProfile {
  return getScreenshotGames(lang)[0];
}

function param(partial: Partial<GameParameter> & Pick<GameParameter, "key" | "title" | "value">): GameParameter {
  return {
    section: "/Script/Engine.GameUserSettings",
    file: "GameUserSettings.ini",
    description: "",
    impact: "Medium",
    category: "Display",
    min: "0",
    max: "4",
    in_game_label: null,
    value_hint: null,
    value_type: "int",
    known: true,
    editable: true,
    present_in_ini: true,
    recommended: null,
    default_value: null,
    tier_hint: null,
    ...partial,
  };
}

export const basicParameters: GameParameter[] = [
  param({ key: "sg.TextureQuality", title: "Texture Quality", value: "3", category: "Scalability", max: "3" }),
  param({ key: "sg.ShadowQuality", title: "Shadow Quality", value: "2", category: "Scalability" }),
  param({ key: "sg.EffectsQuality", title: "Effects Quality", value: "3", category: "Scalability" }),
  param({
    key: "ResolutionSizeX",
    title: "Resolution X",
    value: "2560",
    category: "Display",
    value_type: "int",
    max: "7680",
  }),
  param({
    key: "ResolutionSizeY",
    title: "Resolution Y",
    value: "1440",
    category: "Display",
    value_type: "int",
    max: "4320",
  }),
  param({
    key: "FullscreenMode",
    title: "Window Mode",
    value: "1",
    category: "Display",
    value_type: "int",
    max: "2",
  }),
  param({ key: "bUseVSync", title: "VSync", value: "False", category: "Display", value_type: "bool" }),
];

export const advancedParameters: GameParameter[] = [
  param({
    key: "r.Nanite.MaxPixelsPerEdge",
    title: "Nanite Max Pixels Per Edge",
    file: "Engine.ini",
    section: "/Script/Engine.RendererSettings",
    value: "1.0",
    category: "Rendering",
    value_type: "float",
    tier_hint: "Tier A",
    recommended: "1.0",
  }),
  param({
    key: "r.DLSS.Enable",
    title: "DLSS",
    file: "Engine.ini",
    section: "/Script/Engine.RendererSettings",
    value: "1",
    category: "Upscaling",
    value_type: "bool",
    tier_hint: "Tier A",
  }),
  param({
    key: "r.RayTracing",
    title: "Ray Tracing",
    file: "Engine.ini",
    section: "/Script/Engine.RendererSettings",
    value: "1",
    category: "Rendering",
    value_type: "bool",
    tier_hint: "Tier B",
  }),
  param({
    key: "r.FidelityFX.FSR2.Enable",
    title: "FSR 2",
    file: "Engine.ini",
    section: "/Script/Engine.RendererSettings",
    value: "0",
    category: "Upscaling",
    value_type: "bool",
  }),
  param({
    key: "r.TSR.History.ScreenPercentage",
    title: "TSR Screen Percentage",
    file: "Engine.ini",
    section: "/Script/Engine.RendererSettings",
    value: "100",
    category: "Upscaling",
    value_type: "float",
    value_hint: "UE 5",
  }),
  param({
    key: "r.Lumen.Reflections.Allow",
    title: "Lumen Reflections",
    file: "Engine.ini",
    section: "/Script/Engine.RendererSettings",
    value: "1",
    category: "Lumen",
    value_type: "bool",
    tier_hint: "Tier B",
  }),
];

export const screenshotBackups: BackupInfo[] = [
  {
    id: "2026-06-26_14-32-01",
    created_at: "2026-06-26T14:32:01",
    files: ["GameUserSettings.ini", "Engine.ini", "Scalability.ini"],
  },
  {
    id: "2026-06-25_09-18-44",
    created_at: "2026-06-25T09:18:44",
    files: ["GameUserSettings.ini", "Engine.ini"],
  },
  {
    id: "2026-06-24_21-05-12",
    created_at: "2026-06-24T21:05:12",
    files: ["GameUserSettings.ini"],
  },
];

export const screenshotCategoriesBasic = [
  { cat: "All", count: 54 },
  { cat: "Scalability", count: 13 },
  { cat: "Display", count: 22 },
  { cat: "Window", count: 5 },
  { cat: "GameSpecific", count: 8 },
  { cat: "Audio", count: 4 },
  { cat: "Performance", count: 2 },
];

export const screenshotCategoriesAdvanced = [
  { cat: "All", count: 725 },
  { cat: "Scalability", count: 13 },
  { cat: "Display", count: 29 },
  { cat: "Window", count: 5 },
  { cat: "Rendering", count: 368 },
  { cat: "Shadows", count: 26 },
  { cat: "Textures", count: 32 },
  { cat: "PostProcess", count: 60 },
  { cat: "Graphics", count: 35 },
  { cat: "LOD", count: 22 },
  { cat: "World", count: 15 },
  { cat: "GameSpecific", count: 48 },
  { cat: "Performance", count: 6 },
  { cat: "API", count: 8 },
  { cat: "Jobs", count: 4 },
  { cat: "Audio", count: 2 },
  { cat: "System", count: 1 },
  { cat: "Debug", count: 18 },
  { cat: "Other", count: 33 },
];
