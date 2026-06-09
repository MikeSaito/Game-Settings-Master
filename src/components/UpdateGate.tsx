import { Download, RefreshCw } from "lucide-react";
import type { ReactNode } from "react";
import { useAppUpdater } from "../hooks/useAppUpdater";
import { Button } from "./ui/Button";

interface Props {
  children: ReactNode;
}

export function UpdateGate({ children }: Props) {
  const { status, update, error, progress, retry, installUpdate } =
    useAppUpdater();

  if (status === "ready") {
    return <>{children}</>;
  }

  const progressPercent =
    progress && progress.total > 0
      ? Math.min(100, Math.round((progress.downloaded / progress.total) * 100))
      : null;

  return (
    <div className="app-bg flex min-h-screen flex-col items-center justify-center px-6">
      <div className="w-full max-w-md space-y-6 text-center">
        <div>
          <h1 className="text-xl font-semibold text-[var(--color-text)]">
            Game Settings Master
          </h1>
          <p className="mt-2 text-sm text-muted">
            {status === "checking" && "Проверка обновлений…"}
            {status === "required" &&
              `Доступна новая версия ${update?.version}. Для продолжения нужно обновиться.`}
            {status === "downloading" && "Загрузка и установка обновления…"}
            {status === "error" &&
              (update
                ? "Не удалось установить обновление. Проверьте интернет и повторите."
                : "Не удалось проверить обновления. Проверьте интернет и повторите.")}
          </p>
        </div>

        {status === "checking" && (
          <div className="flex justify-center py-4">
            <span className="h-10 w-10 animate-spin rounded-full border-2 border-[var(--color-border)] border-t-[var(--color-accent)]" />
          </div>
        )}

        {status === "required" && (
          <Button
            variant="primary"
            icon={<Download size={18} />}
            onClick={() => void installUpdate()}
            className="w-full"
          >
            Обновить до {update?.version}
          </Button>
        )}

        {status === "downloading" && (
          <div className="space-y-2">
            <div className="h-2 overflow-hidden rounded-full bg-[var(--color-border)]">
              <div
                className="h-full bg-[var(--color-accent)] transition-all duration-200"
                style={{ width: `${progressPercent ?? 0}%` }}
              />
            </div>
            {progressPercent != null && (
              <p className="text-xs text-muted">{progressPercent}%</p>
            )}
          </div>
        )}

        {status === "error" && (
          <div className="space-y-4">
            {error && (
              <p className="rounded-lg border border-[#5a3030] bg-[#2a1818] px-4 py-3 text-left text-sm text-[#f0a0a0]">
                {error}
              </p>
            )}
            <Button
              variant="primary"
              icon={<RefreshCw size={18} />}
              onClick={() => void retry()}
              className="w-full"
            >
              Повторить
            </Button>
          </div>
        )}
      </div>
    </div>
  );
}
