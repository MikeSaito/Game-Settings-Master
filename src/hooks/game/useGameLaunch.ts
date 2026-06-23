import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { closeGame, launchGame } from "@/lib/api";
import { formatInvokeError } from "@/lib/core";
import type { GameProfile } from "@/lib/core";

export type LaunchStatus = {
  tone: "success" | "warning" | "danger";
  text: string;
} | null;

/** Launch / close game with session guards (ignores stale responses after game switch). */
export function useGameLaunch(game: GameProfile, runningExeName: string | null) {
  const { t } = useTranslation("header");
  const queryClient = useQueryClient();
  const sessionRef = useRef(0);
  const [launchStatus, setLaunchStatus] = useState<LaunchStatus>(null);

  useEffect(() => {
    sessionRef.current += 1;
    setLaunchStatus(null);
  }, [game.id]);

  const launchMutation = useMutation({
    mutationFn: (session: number) => launchGame(game).then((result) => ({ result, session })),
    onSuccess: ({ result, session }) => {
      if (session !== sessionRef.current) return;
      setLaunchStatus({
        tone: result.warning ? "warning" : "success",
        text: result.warning ?? t("launchVia", { launcher: result.launcher }),
      });
      queryClient.invalidateQueries({ queryKey: ["game-running"] });
    },
    onError: (err, session) => {
      if (session !== sessionRef.current) return;
      setLaunchStatus({ tone: "danger", text: formatInvokeError(err) });
    },
  });

  const closeMutation = useMutation({
    mutationFn: (session: number) => {
      if (!runningExeName) return Promise.reject(new Error(t("errors.noProcessName")));
      return closeGame(game.id, runningExeName).then(() => session);
    },
    onSuccess: (session) => {
      if (session !== sessionRef.current) return;
      setLaunchStatus({ tone: "success", text: t("gameClosed") });
      queryClient.invalidateQueries({ queryKey: ["game-running"] });
    },
    onError: (err, session) => {
      if (session !== sessionRef.current) return;
      setLaunchStatus({ tone: "danger", text: formatInvokeError(err) });
    },
  });

  return {
    launchStatus,
    launchMutation,
    closeMutation,
    sessionRef,
  };
}
