import { useQuery } from "@tanstack/react-query";
import { useAppWindowFocused } from "../context/AppWindowFocusProvider";
import { getPresetServerStatus } from "../lib/api";

/** Preset server status — polled on window focus for up-to-date catalog_version. */
export function usePresetServerStatus() {
  const focused = useAppWindowFocused();
  return useQuery({
    queryKey: ["preset-server-status"],
    queryFn: getPresetServerStatus,
    staleTime: 30_000,
    refetchOnWindowFocus: false,
    refetchInterval: focused ? 60_000 : false,
  });
}
