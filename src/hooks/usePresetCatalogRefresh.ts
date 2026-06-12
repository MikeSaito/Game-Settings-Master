import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef } from "react";
import { useAppWindowFocused } from "../context/AppWindowFocusProvider";
import { usePresetServerStatus } from "./usePresetServerStatus";

/** Invalidates presets/parameters only on catalog version change and window focus. */
export function usePresetCatalogRefresh() {
  const queryClient = useQueryClient();
  const lastVersion = useRef<string | null>(null);
  const focused = useAppWindowFocused();
  const { data: status } = usePresetServerStatus();

  const lastSyncAt = useRef<string | null>(null);

  useEffect(() => {
    if (!focused) return;
    const version = status?.catalog_version ?? null;
    const syncAt = status?.last_sync_at ?? null;
    const versionChanged =
      version != null && version.length > 0 && version !== lastVersion.current;
    const syncChanged =
      syncAt != null && syncAt.length > 0 && syncAt !== lastSyncAt.current;
    if (!versionChanged && !syncChanged) return;
    if (version) lastVersion.current = version;
    if (syncAt) lastSyncAt.current = syncAt;
    void queryClient.invalidateQueries({ queryKey: ["presets"] });
    void queryClient.invalidateQueries({ queryKey: ["parameters"] });
  }, [focused, status?.catalog_version, status?.last_sync_at, queryClient]);
}
