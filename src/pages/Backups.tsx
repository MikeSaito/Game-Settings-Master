import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  AlertTriangle,
  History,
  RotateCcw,
  ShieldCheck,
  Trash2,
} from "lucide-react";
import type { ReactNode } from "react";
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Badge } from "../components/ds/Badge";
import { Button } from "../components/ds/Button";
import { Alert, EmptyState } from "../components/ds/Feedback";
import { Panel } from "../components/ds/Panel";
import { formatInvokeError } from "../lib/errors";
import { GameRunningAlert, useGameRunning } from "../hooks/useGameRunning";
import { useRunningExeName } from "../hooks/useRunningExeName";
import { useBackgroundSafeEnabled } from "../hooks/useBackgroundSafeEnabled";
import { listBackups, resetConfigToUser, restoreBackup } from "../lib/api";
import type { BackupInfo, GameProfile } from "../lib/types";

interface Props {
  game: GameProfile | null;
  embedded?: boolean;
}

function formatBackupDate(id: string): string {
  const match = /^(\d{4})(\d{2})(\d{2})_(\d{2})(\d{2})(\d{2})$/.exec(id);
  if (!match) return id;
  const [, y, mo, d, h, mi, s] = match;
  return `${d}.${mo}.${y} · ${h}:${mi}:${s}`;
}

export function Backups({ game, embedded = false }: Props) {
  const { t } = useTranslation("backups");
  const queryClient = useQueryClient();
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
  }, [game?.id]);

  const configDir = game?.config_dir ?? "";
  const activeGameIdRef = useRef(game?.id);
  activeGameIdRef.current = game?.id;
  const runningExeName = useRunningExeName(game);
  const gameRunning = useGameRunning(runningExeName);
  const backupsEnabled = useBackgroundSafeEnabled(!!configDir);

  const { data: backups = [], isLoading, isFetching, refetch } = useQuery({
    queryKey: ["backups", configDir, game?.id],
    queryFn: () => listBackups(configDir, game?.id),
    enabled: backupsEnabled,
    placeholderData: (previousData, previousQuery) =>
      previousQuery?.queryKey?.[2] === game?.id ? previousData : undefined,
  });

  const backupsLoading = (isLoading || isFetching) && backups.length === 0;

  const restore = useMutation({
    mutationFn: (backupId: string) => {
      setRestoringId(backupId);
      const snapshot = {
        gameId: activeGameIdRef.current,
        configDir,
      };
      return restoreBackup(
        configDir,
        backupId,
        runningExeName ?? undefined,
        game?.id,
        game?.engine_family,
        game?.install_dir,
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
      queryClient.invalidateQueries({ queryKey: ["backups", configDir, gameId] });
      queryClient.invalidateQueries({ queryKey: ["preview", configDir] });
      queryClient.invalidateQueries({ queryKey: ["parameters", configDir, gameId] });
      queryClient.invalidateQueries({ queryKey: ["game-config"] });
    },
    onError: (err) => setRestoreError(formatInvokeError(err)),
    onSettled: () => setRestoringId(undefined),
  });

  const reset = useMutation({
    mutationFn: () => {
      const snapshot = {
        gameId: activeGameIdRef.current,
        configDir,
      };
      return resetConfigToUser(
        configDir,
        runningExeName ?? undefined,
        game?.id,
        game?.engine_family,
      ).then((result) => ({ result, snapshot }));
    },
    onMutate: () => {
      setResetError(undefined);
      setSuccessMessage(undefined);
      setResetConfirm(false);
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
      queryClient.invalidateQueries({ queryKey: ["backups", configDir, gameId] });
      queryClient.invalidateQueries({ queryKey: ["preview", configDir] });
      queryClient.invalidateQueries({ queryKey: ["parameters", configDir, gameId] });
      queryClient.invalidateQueries({ queryKey: ["game-config"] });
    },
    onError: (err) => setResetError(formatInvokeError(err)),
  });

  if (!game) {
    return (
      <EmptyState
        icon={History}
        title={t("empty.selectGame")}
        description={t("empty.selectGameDesc")}
      />
    );
  }

  if (!configDir) {
    return (
      <Alert tone="warning" icon={AlertTriangle} title={t("configMissing.title", { name: game.name })}>
        {t("configMissing.body")}
      </Alert>
    );
  }

  return (
    <div className={embedded ? "space-y-6" : "space-y-8"}>
      {!embedded && (
        <div className="flex flex-wrap items-start justify-between gap-4">
          <div className="min-w-0 flex-1">
            <h2 className="text-2xl font-bold tracking-tight text-[var(--color-text)]">
              {t("header.title")}
            </h2>
            <div className="mt-3 flex flex-wrap gap-2">
              <Badge tone="neutral">{t("header.backupsCount", { count: backups.length })}</Badge>
            </div>
          </div>
        </div>
      )}

      {embedded && (
        <div className="flex flex-wrap gap-2">
          <Badge tone="neutral">{t("header.backupsCount", { count: backups.length })}</Badge>
        </div>
      )}

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
        <SectionTitle
          title={t("reset.sectionTitle")}
          description={t("reset.sectionDesc")}
        />
        {resetConfirm ? (
          <Panel padding="md" className="border-[var(--color-danger)]/45 bg-[var(--color-danger-soft)]">
            <p className="text-sm text-[var(--color-text-secondary)]">
              {t("reset.confirmBody")}
            </p>
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
        <SectionTitle
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
            {backups.map((backup: BackupInfo) => (
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

function BackupRow({
  backup,
  restoring,
  disabled,
  onRestore,
}: {
  backup: BackupInfo;
  restoring: boolean;
  disabled: boolean;
  onRestore: () => void;
}) {
  const { t } = useTranslation("backups");
  return (
    <Panel padding="none">
      <div className="flex items-center justify-between gap-4 px-4 py-3">
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-2">
            <span className="text-sm font-medium text-[var(--color-text)]">
              {formatBackupDate(backup.id)}
            </span>
            <span className="font-mono text-xs text-[var(--color-text-muted)]">{backup.id}</span>
          </div>
          <div className="mt-1 flex flex-wrap gap-1.5">
            {backup.files.map((file) => (
              <Badge key={file} tone="neutral">
                {file}
              </Badge>
            ))}
          </div>
        </div>
        <Button
          variant="secondary"
          icon={<RotateCcw size={14} />}
          onClick={onRestore}
          loading={restoring}
          disabled={disabled}
          className="shrink-0 !py-2"
        >
          {t("restore.button")}
        </Button>
      </div>
    </Panel>
  );
}

function SectionTitle({
  title,
  description,
  hint,
}: {
  title: string;
  description?: ReactNode;
  hint?: ReactNode;
}) {
  return (
    <div className="mb-4 flex flex-wrap items-end justify-between gap-2">
      <div className="min-w-0">
        <h3 className="text-sm font-semibold uppercase tracking-wide text-[var(--color-text-secondary)]">
          {title}
        </h3>
        {description && (
          <p className="mt-1 text-sm text-[var(--color-text-muted)]">{description}</p>
        )}
      </div>
      {hint && <div className="text-sm text-[var(--color-text-muted)]">{hint}</div>}
    </div>
  );
}
