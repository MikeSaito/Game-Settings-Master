import { relaunch } from "@tauri-apps/plugin-process";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { useCallback, useEffect, useState } from "react";

export type UpdaterStatus =
  | "checking"
  | "ready"
  | "required"
  | "downloading"
  | "error";

export interface UpdateProgress {
  downloaded: number;
  total: number;
}

const isDev = import.meta.env.DEV;
const UPDATE_CHECK_TIMEOUT_MS = 15_000;
const UPDATE_INSTALL_TIMEOUT_MS = 5 * 60_000;

function withTimeout<T>(promise: Promise<T>, ms: number, message: string): Promise<T> {
  return new Promise((resolve, reject) => {
    const handle = window.setTimeout(() => reject(new Error(message)), ms);
    promise.then(
      (value) => {
        window.clearTimeout(handle);
        resolve(value);
      },
      (err) => {
        window.clearTimeout(handle);
        reject(err);
      },
    );
  });
}

export function useAppUpdater() {
  const [status, setStatus] = useState<UpdaterStatus>(isDev ? "ready" : "checking");
  const [update, setUpdate] = useState<Update | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [progress, setProgress] = useState<UpdateProgress | null>(null);
  const [retryCount, setRetryCount] = useState(0);

  const runCheck = useCallback(async () => {
    if (isDev) {
      setStatus("ready");
      return;
    }

    setStatus("checking");
    setError(null);
    setProgress(null);

    try {
      const found = await withTimeout(
        check(),
        UPDATE_CHECK_TIMEOUT_MS,
        "Update check timed out",
      );
      if (found) {
        setUpdate(found);
        setStatus("required");
      } else {
        setUpdate(null);
        setStatus("ready");
      }
    } catch (err) {
      setError(formatUpdaterError(err));
      setStatus("error");
    }
  }, []);

  useEffect(() => {
    void runCheck();
  }, [runCheck]);

  const installUpdate = useCallback(async () => {
    if (!update) return;

    setStatus("downloading");
    setError(null);
    setProgress(null);

    try {
      await withTimeout(
        update.downloadAndInstall((event) => {
          switch (event.event) {
            case "Started":
              setProgress({
                downloaded: 0,
                total: event.data.contentLength ?? 0,
              });
              break;
            case "Progress":
              setProgress((prev) => ({
                downloaded: (prev?.downloaded ?? 0) + event.data.chunkLength,
                total: prev?.total ?? 0,
              }));
              break;
            case "Finished":
              break;
          }
        }),
        UPDATE_INSTALL_TIMEOUT_MS,
        "Update download timed out",
      );
      await relaunch();
    } catch (err) {
      setError(formatUpdaterError(err));
      setStatus("error");
    }
  }, [update]);

  const retry = useCallback(async () => {
    setRetryCount((prev) => prev + 1);
    await runCheck();
  }, [runCheck]);

  const continueWithoutUpdate = useCallback(() => {
    setStatus("ready");
  }, []);

  return {
    status,
    update,
    error,
    progress,
    retry,
    canBypassOnError: status === "error" && (!update || retryCount > 0),
    continueWithoutUpdate,
    installUpdate,
  };
}

function formatUpdaterError(err: unknown): string {
  if (err instanceof Error) return err.message;
  return String(err);
}
