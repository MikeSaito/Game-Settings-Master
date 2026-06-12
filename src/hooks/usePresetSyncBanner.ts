import { useTranslation } from "react-i18next";
import { formatInvokeError } from "../lib/errors";
import { usePresetServerStatus } from "./usePresetServerStatus";

export interface SyncBanner {
  tone: "info" | "warning" | "error";
  title: string;
  message: string;
}

/** User-facing preset sync status (no URLs or technical details). */
export function usePresetSyncBanner(): SyncBanner | null {
  const { t } = useTranslation("common");
  const { data: status, isLoading, error } = usePresetServerStatus();

  if (isLoading && !status) {
    return {
      tone: "info",
      title: t("presetSyncLoadingTitle"),
      message: t("presetSyncLoadingMessage"),
    };
  }

  if (error) {
    return {
      tone: "warning",
      title: t("presetSyncErrorTitle"),
      message: t("presetSyncErrorMessage"),
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
        title: t("presetSyncOfflineTitle"),
        message: t("presetSyncOfflineMessage", { version: status.catalog_version }),
      };
    }
    return {
      tone: "error",
      title: t("presetSyncNotLoadedTitle"),
      message: formatInvokeError(status.last_sync_error),
    };
  }

  return null;
}
