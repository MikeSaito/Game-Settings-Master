import { RotateCcw } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Badge } from "@/components/ds/Badge";
import { Button } from "@/components/ds/Button";
import { Panel } from "@/components/ds/Panel";
import { formatBackupDate } from "@/lib/backup/formatBackupDate";
import type { BackupInfo } from "@/lib/core";

interface Props {
  backup: BackupInfo;
  restoring: boolean;
  disabled: boolean;
  onRestore: () => void;
}

export function BackupRow({ backup, restoring, disabled, onRestore }: Props) {
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
