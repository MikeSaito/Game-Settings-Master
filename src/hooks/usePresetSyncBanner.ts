import { useQuery } from "@tanstack/react-query";
import { getPresetServerStatus } from "../lib/api";
import { formatInvokeError } from "../lib/errors";

export interface SyncBanner {
  tone: "info" | "warning" | "error";
  title: string;
  message: string;
}

/** Пользовательский статус синхронизации пресетов (без URL и техдеталей). */
export function usePresetSyncBanner(): SyncBanner | null {
  const { data: status, isLoading, error } = useQuery({
    queryKey: ["preset-server-status"],
    queryFn: getPresetServerStatus,
    refetchInterval: 15_000,
  });

  if (isLoading && !status) {
    return {
      tone: "info",
      title: "Загрузка пресетов",
      message: "Синхронизация каталога с сервером…",
    };
  }

  if (error) {
    return {
      tone: "warning",
      title: "Пресеты",
      message: "Не удалось проверить статус синхронизации. Используется локальный кэш, если он есть.",
    };
  }

  if (!status?.configured) {
    return null;
  }

  if (status.last_sync_error && !status.last_sync_ok) {
    const hasCache = !!(
      status.catalog_version && status.cached_packs.length > 0
    );
    if (hasCache) {
      return {
        tone: "warning",
        title: "Офлайн",
        message: `Не удалось обновить пресеты. Используется кэш v${status.catalog_version}.`,
      };
    }
    return {
      tone: "error",
      title: "Пресеты не загружены",
      message: formatInvokeError(status.last_sync_error),
    };
  }

  return null;
}
