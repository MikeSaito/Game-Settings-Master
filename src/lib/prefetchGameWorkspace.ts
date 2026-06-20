import type { QueryClient } from "@tanstack/react-query";
import { getGameParameters, listBackups } from "./api";
import type { AppTab, GameProfile } from "./types";

/** Prefetches data only for the active game tab. */
export function prefetchGameWorkspace(
  queryClient: QueryClient,
  game: GameProfile,
  tab: AppTab,
): void {
  const { config_dir: configDir, engine_family: engineFamily, id, install_dir: installDir } =
    game;

  if (!configDir) return;

  switch (tab) {
    case "advanced":
      void queryClient.prefetchQuery({
        queryKey: ["parameters", configDir, id, engineFamily],
        queryFn: () => getGameParameters(configDir, id, installDir, engineFamily),
        staleTime: 5 * 60_000,
      });
      break;
    case "backups":
      void queryClient.prefetchQuery({
        queryKey: ["backups", configDir, id],
        queryFn: () => listBackups(configDir, id),
      });
      break;
    default:
      break;
  }
}
