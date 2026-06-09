import { useQuery } from "@tanstack/react-query";
import { getPresetServerStatus } from "../lib/api";

/** Статус пресет-сервера — один раз при старте, без фонового опроса. */
export function usePresetServerStatus() {
  return useQuery({
    queryKey: ["preset-server-status"],
    queryFn: getPresetServerStatus,
    staleTime: Infinity,
    refetchOnWindowFocus: false,
    refetchInterval: false,
  });
}
