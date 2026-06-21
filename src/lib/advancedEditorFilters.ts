import {
  ENGINE_CATEGORIES,
  isEngineEnabled,
  isEngineToggleable,
} from "./engineParams";
import type { GameParameter } from "./types";

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

export function filterParamsByCategoryAndSearch(
  visibleParams: GameParameter[],
  activeCategory: string,
  search: string,
  engineEnabled: Set<string>,
): GameParameter[] {
  const q = search.trim().toLowerCase();
  const list = visibleParams.filter((p) => {
    if (activeCategory !== ALL_CATEGORY && p.category !== activeCategory) return false;
    if (!q) return true;
    return (
      p.key.toLowerCase().includes(q) ||
      p.title.toLowerCase().includes(q) ||
      p.description.toLowerCase().includes(q) ||
      (p.value_hint?.toLowerCase().includes(q) ?? false)
    );
  });

  if (!ENGINE_CATEGORIES.has(activeCategory)) {
    return list;
  }

  return [...list].sort((a, b) => {
    const aOn = isEngineEnabled(a, engineEnabled) ? 0 : 1;
    const bOn = isEngineEnabled(b, engineEnabled) ? 0 : 1;
    if (aOn !== bOn) return aOn - bOn;
    return a.title.localeCompare(b.title, "ru");
  });
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
