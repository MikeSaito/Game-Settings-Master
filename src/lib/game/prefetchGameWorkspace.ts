import type { QueryClient } from "@tanstack/react-query";
import { currentLanguage } from "@/i18n";
import { getGameParameters, listBackups } from "@/lib/api";
import { readStoredPanel } from "@/lib/routing/editorPanels";
import type { GameProfile } from "@/lib/core/types";

/** Prefetches data for the active editor panel (from sessionStorage). */
export function prefetchGameWorkspace(queryClient: QueryClient, game: GameProfile): void {
  const {
    config_dir: configDir,
    engine_family: engineFamily,
    engine_version: engineVersion,
    id,
    install_dir: installDir,
  } = game;

  if (!configDir) return;

  const panel = readStoredPanel(id) ?? "basic";

  if (panel === "backups") {
    void queryClient.prefetchQuery({
      queryKey: ["backups", configDir, id],
      queryFn: () => listBackups(configDir, id),
    });
    return;
  }

  void queryClient.prefetchQuery({
    queryKey: [
      "parameters",
      configDir,
      id,
      engineFamily,
      engineVersion,
      currentLanguage(),
    ],
    queryFn: () =>
      getGameParameters(configDir, id, installDir, engineFamily, engineVersion),
    staleTime: 5 * 60_000,
  });
}
