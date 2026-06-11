import { useQuery, useQueryClient } from "@tanstack/react-query";
import { AlertTriangle } from "lucide-react";
import { useEffect, useRef } from "react";
import { usePollingEnabled } from "./useBackgroundSafeEnabled";
import { isGameRunning } from "../lib/api";
import { Alert } from "../components/ui/Alert";

export function useGameRunning(exeName: string | null | undefined): boolean {
  const windowFocused = usePollingEnabled(!!exeName);
  const queryClient = useQueryClient();
  const wasDisabled = useRef(false);

  useEffect(() => {
    if (!windowFocused || !exeName) {
      wasDisabled.current = true;
      return;
    }
    if (!wasDisabled.current) return;
    wasDisabled.current = false;
    void queryClient.invalidateQueries({ queryKey: ["game-running", exeName], exact: true });
    void queryClient.refetchQueries({ queryKey: ["game-running", exeName], exact: true });
  }, [windowFocused, exeName, queryClient]);

  const { data: running = false } = useQuery({
    queryKey: ["game-running", exeName],
    queryFn: () => isGameRunning(exeName!),
    enabled: !!exeName && windowFocused,
    staleTime: 0,
    refetchOnWindowFocus: false,
    refetchInterval: windowFocused ? 4000 : false,
  });
  return running;
}

interface GameRunningAlertProps {
  exeName: string | null | undefined;
  gameName?: string;
  /** reshade — proxy DLL; config (default) — ini-файлы */
  context?: "config" | "reshade";
}

export function GameRunningAlert({
  exeName,
  gameName,
  context = "config",
}: GameRunningAlertProps) {
  const running = useGameRunning(exeName);
  if (!exeName || !running) return null;

  const label = gameName ?? exeName;
  const body =
    context === "reshade"
      ? "Закройте игру перед установкой или удалением ReShade — proxy DLL (dxgi.dll и др.) заблокированы процессом."
      : "Закройте игру перед применением — Engine.ini и другие файлы заблокированы процессом.";

  return (
    <Alert tone="warning" icon={AlertTriangle} title={`${label} запущена`}>
      {body}
    </Alert>
  );
}
