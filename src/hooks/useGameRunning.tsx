import { useQuery, useQueryClient } from "@tanstack/react-query";
import { AlertTriangle } from "lucide-react";
import { useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { usePollingEnabled } from "./useBackgroundSafeEnabled";
import { isGameRunning } from "../lib/api";
import { Alert } from "../components/ds/Feedback";

/** Align with Rust is_exe_running cache TTL (30s foreground). */
const GAME_RUNNING_POLL_MS = 10_000;
const GAME_RUNNING_STALE_MS = 8_000;

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
    staleTime: GAME_RUNNING_STALE_MS,
    refetchOnWindowFocus: false,
    refetchInterval: windowFocused ? GAME_RUNNING_POLL_MS : false,
  });
  return running;
}

interface GameRunningAlertProps {
  exeName: string | null | undefined;
  gameName?: string;
}

export function GameRunningAlert({
  exeName,
  gameName,
}: GameRunningAlertProps) {
  const { t } = useTranslation("errors");
  const running = useGameRunning(exeName);
  if (!exeName || !running) return null;

  const label = gameName ?? exeName;

  return (
    <Alert tone="warning" icon={AlertTriangle} title={t("gameRunningTitle", { label })}>
      {t("gameRunningIni")}
    </Alert>
  );
}
