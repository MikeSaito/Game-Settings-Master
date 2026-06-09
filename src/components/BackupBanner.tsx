import { ShieldCheck } from "lucide-react";
import { Alert } from "./ui/Alert";

interface Props {
  backupId?: string;
  message?: string;
}

export function BackupBanner({ backupId, message }: Props) {
  if (!backupId && !message) return null;

  return (
    <Alert tone="success" icon={ShieldCheck} title="Резервная копия создана">
      {message ??
        `Backup ${backupId}. Откат — во вкладке «Бекапы».`}
    </Alert>
  );
}
