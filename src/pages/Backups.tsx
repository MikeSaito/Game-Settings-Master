import {
  keepPreviousData,
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import {
  AlertTriangle,
  History,
  RotateCcw,
  ShieldCheck,
  Trash2,
} from "lucide-react";
import { useState } from "react";
import { Alert } from "../components/ui/Alert";
import { Badge } from "../components/ui/Badge";
import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import { EmptyState } from "../components/ui/EmptyState";
import { PageHeader } from "../components/ui/PageHeader";
import { SectionHeader } from "../components/ui/SectionHeader";
import { GameRunningAlert, useGameRunning } from "../hooks/useGameRunning";
import { useBackgroundSafeEnabled } from "../hooks/useBackgroundSafeEnabled";
import { listBackups, resetConfigToUser, restoreBackup } from "../lib/api";
import type { BackupInfo, GameProfile } from "../lib/types";

interface Props {
  game: GameProfile | null;
}

function formatBackupDate(id: string): string {
  const match = /^(\d{4})(\d{2})(\d{2})_(\d{2})(\d{2})(\d{2})$/.exec(id);
  if (!match) return id;
  const [, y, mo, d, h, mi, s] = match;
  return `${d}.${mo}.${y} · ${h}:${mi}:${s}`;
}

export function Backups({ game }: Props) {
  const queryClient = useQueryClient();
  const [successMessage, setSuccessMessage] = useState<string>();
  const [restoreError, setRestoreError] = useState<string>();
  const [resetError, setResetError] = useState<string>();
  const [resetConfirm, setResetConfirm] = useState(false);
  const [restoringId, setRestoringId] = useState<string>();

  const configDir = game?.config_dir ?? "";
  const gameRunning = useGameRunning(game?.exe_name);
  const backupsEnabled = useBackgroundSafeEnabled(!!configDir);

  const { data: backups = [], isLoading, isFetching, refetch } = useQuery({
    queryKey: ["backups", configDir],
    queryFn: () => listBackups(configDir),
    enabled: backupsEnabled,
    placeholderData: keepPreviousData,
  });

  const backupsLoading = (isLoading || isFetching) && backups.length === 0;

  const restore = useMutation({
    mutationFn: (backupId: string) => {
      setRestoringId(backupId);
      return restoreBackup(configDir, backupId, game?.exe_name ?? undefined);
    },
    onMutate: () => {
      setRestoreError(undefined);
      setSuccessMessage(undefined);
    },
    onSuccess: (files, backupId) => {
      setSuccessMessage(
        `Восстановлен backup ${formatBackupDate(backupId)}: ${files.join(", ")}. Перезапустите игру.`,
      );
      queryClient.invalidateQueries({ queryKey: ["preview", configDir] });
      queryClient.invalidateQueries({ queryKey: ["parameters", configDir] });
      queryClient.invalidateQueries({ queryKey: ["game-config"] });
    },
    onError: (err) => setRestoreError(String(err)),
    onSettled: () => setRestoringId(undefined),
  });

  const reset = useMutation({
    mutationFn: () => resetConfigToUser(configDir, game?.exe_name ?? undefined),
    onMutate: () => {
      setResetError(undefined);
      setSuccessMessage(undefined);
      setResetConfirm(false);
    },
    onSuccess: (result) => {
      if (result.deleted_files.length === 0) {
        setSuccessMessage(
          "Override-файлы уже отсутствуют — остался только GameUserSettings.ini. Backup создан на всякий случай.",
        );
      } else {
        setSuccessMessage(
          `Сброс выполнен: удалены ${result.deleted_files.join(", ")}. GameUserSettings.ini сохранён. Backup ${result.backup_id}. Перезапустите игру.`,
        );
      }
      queryClient.invalidateQueries({ queryKey: ["backups", configDir] });
      queryClient.invalidateQueries({ queryKey: ["preview", configDir] });
      queryClient.invalidateQueries({ queryKey: ["parameters", configDir] });
      queryClient.invalidateQueries({ queryKey: ["game-config"] });
    },
    onError: (err) => setResetError(String(err)),
  });

  if (!game) {
    return (
      <EmptyState
        icon={History}
        title="Выберите игру"
        description="Откройте библиотеку и выберите игру с config — здесь появится список резервных копий ini."
      />
    );
  }

  if (!configDir) {
    return (
      <Alert tone="warning" icon={AlertTriangle} title={`Config не найден — ${game.name}`}>
        Укажите папку Saved/Config/Windows в библиотеке или запустите игру один раз.
      </Alert>
    );
  }

  return (
    <div className="space-y-8">
      <PageHeader
        title="Бекапы"
        meta={<Badge tone="default">{backups.length} копий</Badge>}
      />

      <Alert tone="info" title="Как это работает">
        Перед каждым применением пресета или ручных правок создаётся снимок ini-файлов в папке
        `.uesm-backups` рядом с конфигами.
      </Alert>

      <GameRunningAlert exeName={game.exe_name} gameName={game.name} />

      {restoreError && (
        <Alert tone="error" title="Ошибка восстановления">
          {restoreError}
        </Alert>
      )}

      {resetError && (
        <Alert tone="error" title="Ошибка сброса">
          {resetError}
        </Alert>
      )}

      {successMessage && (
        <Alert tone="success" icon={ShieldCheck} title="Готово">
          {successMessage}
        </Alert>
      )}

      <section>
        <SectionHeader
          title="Сброс до пользовательских"
          description="Удаляет Engine.ini, Game.ini, Scalability.ini и Input.ini. GameUserSettings.ini не трогается."
        />
        {resetConfirm ? (
          <Card padding="md" className="border-[#5a3030] bg-[#2a1818]">
            <p className="text-sm text-[var(--color-text-secondary)]">
              Перед сбросом создаётся backup. Игра должна быть закрыта. Продолжить?
            </p>
            <div className="mt-4 flex flex-wrap gap-3">
              <Button
                variant="danger"
                icon={<Trash2 size={16} />}
                onClick={() => reset.mutate()}
                loading={reset.isPending}
                disabled={gameRunning}
              >
                Да, сбросить
              </Button>
              <Button variant="secondary" onClick={() => setResetConfirm(false)} disabled={reset.isPending}>
                Отмена
              </Button>
            </div>
          </Card>
        ) : (
          <Button
            variant="danger"
            icon={<Trash2 size={16} />}
            onClick={() => setResetConfirm(true)}
            disabled={gameRunning || restore.isPending}
          >
            Сброс (только GameUserSettings.ini)
          </Button>
        )}
      </section>

      <section>
        <SectionHeader
          title="Список резервных копий"
          description="Новые сверху."
          hint={
            <Button variant="ghost" className="!px-2 !py-1 text-xs" onClick={() => refetch()}>
              Обновить
            </Button>
          }
        />

        {backupsLoading ? (
          <Card padding="md">
            <div className="flex flex-col items-center gap-3 py-6">
              <span className="h-8 w-8 animate-spin rounded-full border-2 border-[var(--color-border)] border-t-[var(--color-accent)]" />
              <p className="text-sm text-muted">Загрузка списка…</p>
            </div>
          </Card>
        ) : backups.length === 0 ? (
          <EmptyState
            icon={History}
            title="Пока нет бекапов"
            description="Примените пресет или сохраните ручные правки — первая копия появится автоматически."
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
  return (
    <Card key={backup.id} padding="sm" className="!p-0">
      <div className="flex items-center justify-between gap-4 px-4 py-3">
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-2">
            <span className="text-sm font-medium text-[var(--color-text)]">
              {formatBackupDate(backup.id)}
            </span>
            <span className="font-mono text-xs text-muted">{backup.id}</span>
          </div>
          <div className="mt-1 flex flex-wrap gap-1.5">
            {backup.files.map((file) => (
              <Badge key={file} tone="default">
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
          Восстановить
        </Button>
      </div>
    </Card>
  );
}
