import type { QueryClient } from "@tanstack/react-query";

/** Refresh editor-related queries after ini-changing operations. */
export function invalidateGameWorkspace(
  queryClient: QueryClient,
  configDir: string,
  gameId: string,
): void {
  void queryClient.invalidateQueries({ queryKey: ["backups", configDir, gameId] });
  void queryClient.invalidateQueries({ queryKey: ["parameters", configDir, gameId] });
  void queryClient.invalidateQueries({ queryKey: ["game-config"] });
}
