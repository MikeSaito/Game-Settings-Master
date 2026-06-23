import type { GameParameter } from "@/lib/core/types";

export type ParamPatch = Pick<GameParameter, "key" | "section" | "file" | "value">;
