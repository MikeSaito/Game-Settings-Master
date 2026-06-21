import { exeNameForRunningCheck } from "../lib/gameRunning";
import type { GameProfile } from "../lib/types";

/** Resolved exe name for is_game_running. */
export function useRunningExeName(
  game: GameProfile | null | undefined,
): string | null | undefined {
  if (!game) return undefined;
  return exeNameForRunningCheck(game.exe_name, undefined) ?? undefined;
}
