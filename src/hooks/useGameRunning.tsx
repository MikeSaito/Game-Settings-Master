import { useQuery, useQueryClient } from "@tanstack/react-query";
import { AlertTriangle } from "lucide-react";
import { useEffect, useRef } from "react";
import { usePollingEnabled } from "./useBackgroundSafeEnabled";
import { isGameRunning } from "../lib/api";
import { Alert } from "../components/ui/Alert";

const REFOCUS_CHECK_MS = 800;

export function useGameRunning(exeName: string | null | undefined): boolean {
  const windowFocused = usePollingEnabled(!!exeName);
  const queryClient = useQueryClient();
  const wasBackground = useRef(false);

  useEffect(() => {
    if (!windowFocused || !exeName) {
      wasBackground.current = true;
      return;
    }
    if (!wasBackground.current) return;
    wasBackground.current = false;

    const timer = window.setTimeout(() => {
      void queryClient.invalidateQueries({ queryKey: ["game-running", exeName] });
    }, REFOCUS_CHECK_MS);

    return () => window.clearTimeout(timer);
  }, [windowFocused, exeName, queryClient]);

  const { data: running = false } = useQuery({
    queryKey: ["game-running", exeName],
    queryFn: () => isGameRunning(exeName!),
    enabled: !!exeName && windowFocused,
    staleTime: Infinity,
    refetchOnWindowFocus: false,
    refetchInterval: false,
  });
  return running;
}

interface GameRunningAlertProps {
  exeName: string | null | undefined;
  gameName?: string;
}

export function GameRunningAlert({ exeName, gameName }: GameRunningAlertProps) {
  const running = useGameRunning(exeName);
  if (!exeName || !running) return null;

  const label = gameName ?? exeName;
  return (
    <Alert tone="warning" icon={AlertTriangle} title={`${label} запущена`}>
      Закройте игру перед применением — Engine.ini и другие файлы заблокированы процессом.
    </Alert>
  );
}
