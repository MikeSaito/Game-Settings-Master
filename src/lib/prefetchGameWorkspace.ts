import type { QueryClient } from "@tanstack/react-query";
import {
  getGameConfig,
  getGameParameters,
  getReShadeWorkspace,
  listBackups,
  listPresets,
} from "./api";
import type { AppTab, GameProfile } from "./types";

/** Prefetches data only for the active game tab. */
export function prefetchGameWorkspace(
  queryClient: QueryClient,
  game: GameProfile,
  tab: AppTab,
): void {
  const { config_dir: configDir, engine_family: engineFamily, id, install_dir: installDir } =
    game;

  if (!configDir && tab !== "wizard" && tab !== "reshade") return;

  switch (tab) {
    case "wizard":
      if (engineFamily && engineFamily !== "unknown") {
        void queryClient.prefetchQuery({
          queryKey: ["presets", engineFamily, id],
          queryFn: () => listPresets(engineFamily, id),
          staleTime: 10 * 60_000,
        });
      }
      if (configDir) {
        void queryClient.prefetchQuery({
          queryKey: ["game-config", configDir, id, engineFamily],
          queryFn: () => getGameConfig(configDir, id, engineFamily),
          staleTime: 5 * 60_000,
        });
      }
      break;
    case "advanced":
      if (configDir) {
        void queryClient.prefetchQuery({
          queryKey: ["parameters", configDir, id, engineFamily],
          queryFn: () => getGameParameters(configDir, id, installDir, engineFamily),
          staleTime: 5 * 60_000,
        });
      }
      break;
    case "backups":
      if (configDir) {
        void queryClient.prefetchQuery({
          queryKey: ["backups", configDir, id],
          queryFn: () => listBackups(configDir, id),
        });
      }
      break;
    case "reshade":
      void queryClient.prefetchQuery({
        queryKey: ["reshade-workspace", id],
        queryFn: () => getReShadeWorkspace(game),
        staleTime: 10_000,
      });
      break;
    default:
      break;
  }
}
