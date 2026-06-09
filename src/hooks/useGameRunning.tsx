import { useQuery } from "@tanstack/react-query";
import { AlertTriangle } from "lucide-react";
import { isGameRunning } from "../lib/api";
import { Alert } from "../components/ui/Alert";

const POLL_MS = 3000;

export function useGameRunning(exeName: string | null | undefined): boolean {
  const { data: running = false } = useQuery({
    queryKey: ["game-running", exeName],
    queryFn: () => isGameRunning(exeName!),
    enabled: !!exeName,
    refetchInterval: POLL_MS,
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
