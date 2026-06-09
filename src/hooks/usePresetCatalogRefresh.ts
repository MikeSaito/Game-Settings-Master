import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef } from "react";
import { useAppWindowFocused } from "../context/AppWindowFocusProvider";
import { usePresetServerStatus } from "./usePresetServerStatus";

/** Инвалидирует пресеты/параметры только при смене версии каталога и фокусе окна. */
export function usePresetCatalogRefresh() {
  const queryClient = useQueryClient();
  const lastVersion = useRef<string | null>(null);
  const focused = useAppWindowFocused();
  const { data: status } = usePresetServerStatus();

  useEffect(() => {
    if (!focused) return;
    const version = status?.catalog_version ?? null;
    if (!version || version === lastVersion.current) return;
    lastVersion.current = version;
    void queryClient.invalidateQueries({ queryKey: ["presets"] });
    void queryClient.invalidateQueries({ queryKey: ["parameters"] });
  }, [focused, status?.catalog_version, queryClient]);
}
