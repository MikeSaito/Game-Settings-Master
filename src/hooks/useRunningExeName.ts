import { useQuery } from "@tanstack/react-query";
import { getReShadeStatus } from "../lib/api";
import { exeNameForRunningCheck } from "../lib/gameRunning";
import { supportsReShade } from "../lib/gameEngine";
import type { GameProfile } from "../lib/types";
import { useBackgroundSafeEnabled } from "./useBackgroundSafeEnabled";

/** Resolved exe name for is_game_running (profile exe_name or backend exe_path). */
export function useRunningExeName(
  game: GameProfile | null | undefined,
): string | null | undefined {
  const queriesEnabled = useBackgroundSafeEnabled(!!game);
  const needsStatus = !!game && (!game.exe_name || supportsReShade(game));

  const { data: status } = useQuery({
    queryKey: ["reshade-status", game?.id],
    queryFn: () => getReShadeStatus(game!),
    enabled: queriesEnabled && needsStatus,
    staleTime: 30_000,
    retry: 1,
  });

  if (!game) return undefined;
  return exeNameForRunningCheck(game.exe_name, status?.exe_path) ?? undefined;
}
