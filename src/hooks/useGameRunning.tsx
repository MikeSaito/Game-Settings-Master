import { useQuery, useQueryClient } from "@tanstack/react-query";
import { AlertTriangle } from "lucide-react";
import { useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
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
  /** reshade — proxy DLL; config (default) — ini files */
  context?: "config" | "reshade";
}

export function GameRunningAlert({
  exeName,
  gameName,
  context = "config",
}: GameRunningAlertProps) {
  const { t } = useTranslation("errors");
  const running = useGameRunning(exeName);
  if (!exeName || !running) return null;

  const label = gameName ?? exeName;
  const body =
    context === "reshade" ? t("gameRunningReshade") : t("gameRunningIni");

  return (
    <Alert tone="warning" icon={AlertTriangle} title={t("gameRunningTitle", { label })}>
      {body}
    </Alert>
  );
}
