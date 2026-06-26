import { useMutation, useQueryClient } from "@tanstack/react-query";
import type { MutableRefObject } from "react";
import { useTranslation } from "react-i18next";
import { resetConfigToUser, restoreBackup } from "@/lib/api";
import { formatInvokeError } from "@/lib/core";
import { formatBackupDate } from "@/lib/backup/formatBackupDate";
import { invalidateGameWorkspace } from "@/lib/game/invalidateGameWorkspace";
import type { GameProfile } from "@/lib/core";

interface Options {
  game: GameProfile;
  configDir: string;
  runningExeName: string | null;
  activeGameIdRef: MutableRefObject<string | undefined>;
  setSuccessMessage: (message: string | undefined) => void;
  setRestoreError: (error: string | undefined) => void;
  setResetError: (error: string | undefined) => void;
  setRestoringId: (id: string | undefined) => void;
  onResetConfirmClose: () => void;
}

export function useBackupMutations({
  game,
  configDir,
  runningExeName,
  activeGameIdRef,
  setSuccessMessage,
  setRestoreError,
  setResetError,
  setRestoringId,
  onResetConfirmClose,
}: Options) {
  const { t } = useTranslation("backups");
  const queryClient = useQueryClient();

  const restore = useMutation({
    mutationFn: (backupId: string) => {
      setRestoringId(backupId);
      const snapshot = { gameId: activeGameIdRef.current, configDir };
      return restoreBackup(
        configDir,
        backupId,
        runningExeName ?? undefined,
        game.id,
        game.engine_family,
        game.install_dir,
      ).then((files) => ({ files, backupId, snapshot }));
    },
    onMutate: () => {
      setRestoreError(undefined);
      setSuccessMessage(undefined);
    },
    onSuccess: ({ files, backupId, snapshot }) => {
      const gameId = activeGameIdRef.current;
      if (!gameId || gameId !== snapshot.gameId || configDir !== snapshot.configDir) return;
      setSuccessMessage(
        t("restore.success", {
          date: formatBackupDate(backupId),
          files: files.join(", "),
        }),
      );
      invalidateGameWorkspace(queryClient, configDir, gameId);
    },
    onError: (err) => setRestoreError(formatInvokeError(err)),
    onSettled: () => setRestoringId(undefined),
  });

  const reset = useMutation({
    mutationFn: () => {
      const snapshot = { gameId: activeGameIdRef.current, configDir };
      return resetConfigToUser(
        configDir,
        runningExeName ?? undefined,
        game.id,
        game.engine_family,
      ).then((result) => ({ result, snapshot }));
    },
    onMutate: () => {
      setResetError(undefined);
      setSuccessMessage(undefined);
      onResetConfirmClose();
    },
    onSuccess: ({ result, snapshot }) => {
      const gameId = activeGameIdRef.current;
      if (!gameId || gameId !== snapshot.gameId || configDir !== snapshot.configDir) return;
      if (result.deleted_files.length === 0) {
        setSuccessMessage(t("reset.noFiles"));
      } else {
        setSuccessMessage(
          t("reset.success", {
            files: result.deleted_files.join(", "),
            backupId: result.backup_id,
          }),
        );
      }
      invalidateGameWorkspace(queryClient, configDir, gameId);
    },
    onError: (err) => setResetError(formatInvokeError(err)),
  });

  return { restore, reset };
}
