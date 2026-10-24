import type { GameParameter } from "@/lib/core";
import type { AppLanguage } from "@/i18n";
import { basicParameters } from "./fixtures";

// Small, fictional catalog used only by the README capture mode.
export function readmeParameters(language: AppLanguage): GameParameter[] {
  const ru = language === "ru";
  const titles: Record<string, string> = {
    "sg.TextureQuality": "Качество текстур", "sg.ShadowQuality": "Качество теней",
    "sg.EffectsQuality": "Качество эффектов", ResolutionSizeX: "Ширина экрана",
    ResolutionSizeY: "Высота экрана", FullscreenMode: "Режим окна", bUseVSync: "VSync",
  };
  const parameters = basicParameters.map((param) => ({
    ...param, catalog_recommended: true, title: ru ? titles[param.key] ?? param.title : param.title,
    description: ru
      ? "Демонстрационный параметр пользовательских настроек. Доступные значения и их поведение зависят от игры."
      : "Demonstration game setting. Available values and their behavior depend on the game.",
  }));
  const template = parameters[0];
  return [...parameters,
    {
      ...template, key: "r.Tonemapper.Sharpen", value: "0.5", value_type: "float",
      min: "0", max: "10", category: "PostProcess", file: "Engine.ini", section: "SystemSettings",
      title: ru ? "Повышение резкости" : "Sharpening",
      description: ru
        ? "Управляет повышением резкости изображения на этапе тонального отображения. Поддержка зависит от используемого игрой рендеринга."
        : "Controls image sharpening in the tonemapping pass. Support depends on the rendering pipeline used by the game.",
    },
    {
      ...template, key: "r.Lumen.Reflections.Allow", value: "1", value_type: "bool",
      min: "0", max: "1", category: "Rendering", file: "Engine.ini", section: "SystemSettings",
      title: ru ? "Отражения Lumen" : "Lumen reflections",
      description: ru ? "Разрешает отражения Lumen, если игра использует эту систему." : "Allows Lumen reflections when the game uses this system.",
    },
    {
      ...template, key: "r.ViewDistanceScale", value: "1", value_type: "float",
      min: "0.1", max: "3", category: "Rendering", file: "Engine.ini", section: "SystemSettings",
      title: ru ? "Дальность отображения" : "View distance scale",
      description: ru ? "Множитель дальности отображения объектов с настроенным расстоянием отсечения." : "View distance multiplier for objects with a configured draw distance.",
    },
  ];
}
