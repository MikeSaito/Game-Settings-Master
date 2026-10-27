import { Download, RefreshCw } from "lucide-react";
import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { useAppUpdater } from "@/hooks/app/useAppUpdater";
import { Button } from "@/components/ds/Button";
import { StartupScreen } from "./StartupScreen";

interface Props {
  children: ReactNode;
}

export function UpdateGate({ children }: Props) {
  const { t } = useTranslation("updater");
  const {
    status,
    update,
    error,
    progress,
    retry,
    installUpdate,
    canBypassOnError,
    continueWithoutUpdate,
  } = useAppUpdater();

  if (status === "ready") {
    return <>{children}</>;
  }

  const progressPercent =
    progress && progress.total > 0
      ? Math.min(100, Math.round((progress.downloaded / progress.total) * 100))
      : null;

  return (
    <StartupScreen
      stage="updates"
      title={
        status === "checking"
          ? t("checking")
          : status === "required"
            ? t("required", { version: update?.version })
            : status === "downloading"
              ? t("downloading")
              : status === "installing"
                ? t("installing")
                : update
                  ? t("installFailed")
                  : t("checkFailed")
      }
      description={
        status === "checking"
          ? undefined
          : t(
              status === "error"
                ? "errorHint"
                : status === "required"
                  ? "requiredHint"
                  : "downloadHint",
            )
      }
      busy={
        status === "checking" ||
        status === "downloading" ||
        status === "installing"
      }
      progress={status === "downloading" ? progressPercent : null}
    >
      {status === "required" && (
        <Button
          variant="primary"
          icon={<Download size={18} />}
          onClick={() => void installUpdate()}
          className="w-full"
        >
          {t("updateTo", { version: update?.version })}
        </Button>
      )}

      {status === "downloading" && progressPercent != null && (
        <p className="text-sm text-[var(--color-text-muted)]">
          {progressPercent}%
        </p>
      )}

      {status === "error" && (
        <div className="space-y-4">
          {error && (
            <p className="startup-error" role="alert">
              {error}
            </p>
          )}
          <Button
            variant="primary"
            icon={<RefreshCw size={18} />}
            onClick={() => void retry()}
            className="w-full"
          >
            {t("retry")}
          </Button>
          {canBypassOnError && (
            <Button
              variant="secondary"
              onClick={continueWithoutUpdate}
              className="w-full"
            >
              {t("continueOffline")}
            </Button>
          )}
        </div>
      )}
    </StartupScreen>
  );
}
