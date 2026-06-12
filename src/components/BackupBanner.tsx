import { ShieldCheck } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Alert } from "./ui/Alert";

interface Props {
  backupId?: string;
  message?: string;
}

export function BackupBanner({ backupId, message }: Props) {
  const { t } = useTranslation("backups");
  if (!backupId && !message) return null;

  return (
    <Alert tone="success" icon={ShieldCheck} title={t("banner.title")}>
      {message ?? t("banner.fallback", { backupId })}
    </Alert>
  );
}
