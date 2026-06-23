import { exeNameForRunningCheck } from "@/lib/game";
import type { GameProfile } from "@/lib/core";

/** Resolved exe name for is_game_running. */
export function useRunningExeName(
  game: GameProfile | null | undefined,
): string | null {
  if (!game) return null;
  return exeNameForRunningCheck(game.exe_name, undefined) ?? null;
}
