import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef } from "react";
import { getPresetServerStatus } from "../lib/api";

/** Инвалидирует пресеты/параметры после фонового sync (Rust startup). */
export function usePresetCatalogRefresh() {
  const queryClient = useQueryClient();
  const lastVersion = useRef<string | null>(null);

  const { data: status } = useQuery({
    queryKey: ["preset-server-status"],
    queryFn: getPresetServerStatus,
    refetchInterval: 15_000,
  });

  useEffect(() => {
    const version = status?.catalog_version ?? null;
    if (!version || version === lastVersion.current) return;
    lastVersion.current = version;
    void queryClient.invalidateQueries({ queryKey: ["presets"] });
    void queryClient.invalidateQueries({ queryKey: ["parameters"] });
  }, [status?.catalog_version, queryClient]);
}
