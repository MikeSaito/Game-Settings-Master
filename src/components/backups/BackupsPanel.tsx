import { AlertTriangle, History, ShieldCheck, Trash2 } from "lucide-react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useQuery } from "@tanstack/react-query";
import { BackupRow } from "@/components/backups/BackupRow";
import { BackupSectionTitle } from "@/components/backups/BackupSectionTitle";
import { OVERRIDE_INI_FILES_LABEL } from "@/lib/ini/configFiles";
import { Badge } from "@/components/ds/Badge";
import { Button } from "@/components/ds/Button";
import { Alert, EmptyState } from "@/components/ds/Feedback";
import { Panel } from "@/components/ds/Panel";
import { ConfigPathHelp } from "@/components/library/ConfigPathHelp";
import { useBackgroundSafeEnabled } from "@/hooks/app/useBackgroundSafeEnabled";
import { useActiveGameIdRef } from "@/hooks/game/useActiveGameIdRef";
import { useBackupMutations } from "@/hooks/game/useBackupMutations";
import { GameRunningAlert, useGameRunning } from "@/hooks/game/useGameRunning";
import { useRunningExeName } from "@/hooks/game/useRunningExeName";
import { listBackups } from "@/lib/api";
import type { GameProfile } from "@/lib/core";

interface Props {
  game: GameProfile;
}

export function BackupsPanel({ game }: Props) {
  const { t } = useTranslation("backups");
  const [successMessage, setSuccessMessage] = useState<string>();
  const [restoreError, setRestoreError] = useState<string>();
  const [resetError, setResetError] = useState<string>();
  const [resetConfirm, setResetConfirm] = useState(false);
  const [restoringId, setRestoringId] = useState<string>();

  useEffect(() => {
    setSuccessMessage(undefined);
    setRestoreError(undefined);
    setResetError(undefined);
    setResetConfirm(false);
    setRestoringId(undefined);
  }, [game.id]);

  const configDir = game.config_dir ?? "";
  const activeGameIdRef = useActiveGameIdRef(game.id);
  const runningExeName = useRunningExeName(game);
  const gameRunning = useGameRunning(runningExeName);
  const backupsEnabled = useBackgroundSafeEnabled(!!configDir);

  const { data: backups = [], isLoading, isFetching, refetch } = useQuery({
    queryKey: ["backups", configDir, game.id],
    queryFn: () => listBackups(configDir, game.id),
    enabled: backupsEnabled,
    placeholderData: (previousData, previousQuery) =>
      previousQuery?.queryKey?.[2] === game.id ? previousData : undefined,
  });

  const backupsLoading = (isLoading || isFetching) && backups.length === 0;

  const { restore, reset } = useBackupMutations({
    game,
    configDir,
    runningExeName: runningExeName ?? null,
    activeGameIdRef,
    setSuccessMessage,
    setRestoreError,
    setResetError,
    setRestoringId,
    onResetConfirmClose: () => setResetConfirm(false),
  });

  if (!configDir) {
    return (
      <div className="space-y-3">
        <Alert tone="warning" icon={AlertTriangle} title={t("configMissing.title", { name: game.name })}>
          {t("configMissing.body")}
        </Alert>
        <ConfigPathHelp />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-wrap gap-2">
        <Badge tone="neutral">{t("header.backupsCount", { count: backups.length })}</Badge>
      </div>

      <Alert tone="info" title={t("howItWorks.title")}>
        {t("howItWorks.body")}
      </Alert>

      <GameRunningAlert exeName={runningExeName} gameName={game.name} />

      {restoreError && (
        <Alert tone="danger" title={t("restore.errorTitle")}>
          {restoreError}
        </Alert>
      )}

      {resetError && (
        <Alert tone="danger" title={t("reset.errorTitle")}>
          {resetError}
        </Alert>
      )}

      {successMessage && (
        <Alert tone="success" icon={ShieldCheck} title={t("successTitle")}>
          {successMessage}
        </Alert>
      )}

      <section>
        <BackupSectionTitle
          title={t("reset.sectionTitle")}
          description={t("reset.sectionDesc", { files: OVERRIDE_INI_FILES_LABEL })}
        />
        {resetConfirm ? (
          <Panel padding="md" className="border-[var(--color-danger)]/45 bg-[var(--color-danger-soft)]">
            <p className="text-sm text-[var(--color-text-secondary)]">{t("reset.confirmBody")}</p>
            <div className="mt-4 flex flex-wrap gap-3">
              <Button
                variant="danger"
                icon={<Trash2 size={16} />}
                onClick={() => reset.mutate()}
                loading={reset.isPending}
                disabled={gameRunning}
              >
                {t("reset.confirmYes")}
              </Button>
              <Button variant="secondary" onClick={() => setResetConfirm(false)} disabled={reset.isPending}>
                {t("reset.cancel")}
              </Button>
            </div>
          </Panel>
        ) : (
          <Button
            variant="danger"
            icon={<Trash2 size={16} />}
            onClick={() => setResetConfirm(true)}
            disabled={gameRunning || restore.isPending}
          >
            {t("reset.button")}
          </Button>
        )}
      </section>

      <section>
        <BackupSectionTitle
          title={t("list.title")}
          description={t("list.desc")}
          hint={
            <Button variant="ghost" size="sm" onClick={() => refetch()}>
              {t("list.refresh")}
            </Button>
          }
        />

        {backupsLoading ? (
          <Panel padding="md">
            <div className="flex flex-col items-center gap-3 py-6">
              <span className="h-8 w-8 animate-spin rounded-full border-2 border-[var(--color-border)] border-t-[var(--color-accent)]" />
              <p className="text-sm text-[var(--color-text-muted)]">{t("list.loading")}</p>
            </div>
          </Panel>
        ) : backups.length === 0 ? (
          <EmptyState
            icon={History}
            title={t("list.emptyTitle")}
            description={t("list.emptyDesc")}
          />
        ) : (
          <div className="space-y-2">
            {backups.map((backup) => (
              <BackupRow
                key={backup.id}
                backup={backup}
                restoring={restore.isPending && restoringId === backup.id}
                disabled={gameRunning || restore.isPending}
                onRestore={() => restore.mutate(backup.id)}
              />
            ))}
          </div>
        )}
      </section>
    </div>
  );
}
