import {
  ENGINE_CATEGORIES,
  isEngineEnabled,
  isEngineToggleable,
} from "./engineParams";
import type { GameParameter } from "../core/types";

export const ALL_CATEGORY = "All";

const GAME_RENDERING_KEY_MARKERS = [
  "dlss",
  "xess",
  "fsr",
  "tsr",
  "raytracing",
  "ray_tracing",
  "lumen",
  "nanite",
  "upscal",
  "framegeneration",
  "frame_generation",
];

export const CATEGORY_ORDER = [
  "Scalability",
  "Graphics",
  "Display",
  "Window",
  "API",
  "Jobs",
  "Rendering",
  "Shadows",
  "Textures",
  "PostProcess",
  "Mirrors",
  "LOD",
  "World",
  "Media",
  "Startup",
  "GameSpecific",
  "Audio",
  "Performance",
  "System",
  "Debug",
  "Other",
] as const;

export function normalizeParameterCategory(param: GameParameter): GameParameter {
  const key = param.key.toLowerCase();
  if (GAME_RENDERING_KEY_MARKERS.some((marker) => key.includes(marker))) {
    return param.category === "Rendering" ? param : { ...param, category: "Rendering" };
  }
  if (param.category === "AuthorCurated") {
    return { ...param, category: "GameSpecific" };
  }
  return param;
}

export function normalizeParameterCategories(params: GameParameter[]): GameParameter[] {
  return params.map(normalizeParameterCategory);
}

export function buildCategoryList(
  visibleParams: GameParameter[],
): { cat: string; count: number }[] {
  const counts = new Map<string, number>();
  for (const p of visibleParams) {
    counts.set(p.category, (counts.get(p.category) ?? 0) + 1);
  }
  const ordered = CATEGORY_ORDER.filter((c) => counts.has(c));
  for (const c of counts.keys()) {
    if (!ordered.includes(c as (typeof CATEGORY_ORDER)[number])) {
      ordered.push(c as (typeof CATEGORY_ORDER)[number]);
    }
  }
  return [
    { cat: ALL_CATEGORY, count: visibleParams.length },
    ...ordered.map((cat) => ({ cat, count: counts.get(cat) ?? 0 })),
  ];
}

const searchTextCache = new WeakMap<GameParameter, string>();

export function parameterSearchText(param: GameParameter): string {
  const cached = searchTextCache.get(param);
  if (cached) return cached;
  const text = [
    param.key,
    param.title,
    param.description,
    param.value_hint ?? "",
    param.category,
    param.file,
  ]
    .join("\n")
    .toLowerCase();
  searchTextCache.set(param, text);
  return text;
}

export function filterParamsByCategory(
  visibleParams: GameParameter[],
  activeCategory: string,
): GameParameter[] {
  if (activeCategory === ALL_CATEGORY) return visibleParams;
  return visibleParams.filter((p) => p.category === activeCategory);
}

export function filterParamsBySearch(
  visibleParams: GameParameter[],
  search: string,
): GameParameter[] {
  const q = search.trim().toLowerCase();
  if (!q) return visibleParams;
  return visibleParams.filter((p) => parameterSearchText(p).includes(q));
}

export function shouldSortByEngineEnabled(activeCategory: string): boolean {
  return ENGINE_CATEGORIES.has(activeCategory);
}

export function sortParamsForEngineCategory(
  visibleParams: GameParameter[],
  activeCategory: string,
  engineEnabled: Set<string>,
): GameParameter[] {
  if (!shouldSortByEngineEnabled(activeCategory)) {
    return visibleParams;
  }

  return [...visibleParams].sort((a, b) => {
    const aOn = isEngineEnabled(a, engineEnabled) ? 0 : 1;
    const bOn = isEngineEnabled(b, engineEnabled) ? 0 : 1;
    if (aOn !== bOn) return aOn - bOn;
    return a.title.localeCompare(b.title, "ru");
  });
}

export function filterParamsByCategoryAndSearch(
  visibleParams: GameParameter[],
  activeCategory: string,
  search: string,
  engineEnabled: Set<string>,
): GameParameter[] {
  const categoryFiltered = filterParamsByCategory(visibleParams, activeCategory);
  const searched = filterParamsBySearch(categoryFiltered, search);
  return sortParamsForEngineCategory(searched, activeCategory, engineEnabled);
}

export function countEngineStats(
  visibleParams: GameParameter[],
  engineEnabled: Set<string>,
): { total: number; on: number; off: number } {
  const engine = visibleParams.filter(
    (p) => p.file === "Engine.ini" && isEngineToggleable(p),
  );
  const on = engine.filter((p) => isEngineEnabled(p, engineEnabled)).length;
  return { total: engine.length, on, off: engine.length - on };
}

export function paramRowKey(param: Pick<GameParameter, "file" | "section" | "key">): string {
  return `${param.file}-${param.section}-${param.key}`;
}
