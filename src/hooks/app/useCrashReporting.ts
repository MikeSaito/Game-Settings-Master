import { useEffect } from "react";
import { submitCrashReport } from "@/lib/api";
import { buildCrashReportPayload } from "@/lib/crashReport";
import { loadAppSettings } from "@/lib/settings";

function crashReportsEnabled(): boolean {
  return loadAppSettings().crashReportsEnabled;
}

async function recordCrash(
  kind: "uncaught" | "unhandled_rejection",
  error: unknown,
): Promise<void> {
  if (!crashReportsEnabled()) return;
  try {
    await submitCrashReport(buildCrashReportPayload(kind, error));
  } catch (reportError) {
    console.error("Failed to submit crash report:", reportError);
  }
}

/** Opt-in global handlers — no analytics, only explicit crash payloads. */
export function useCrashReporting(): void {
  useEffect(() => {
    const onError = (event: ErrorEvent) => {
      void recordCrash("uncaught", event.error ?? event.message);
    };
    const onRejection = (event: PromiseRejectionEvent) => {
      void recordCrash("unhandled_rejection", event.reason);
    };

    window.addEventListener("error", onError);
    window.addEventListener("unhandledrejection", onRejection);
    return () => {
      window.removeEventListener("error", onError);
      window.removeEventListener("unhandledrejection", onRejection);
    };
  }, []);
}

export async function reportBoundaryError(
  error: Error,
  componentStack?: string | null,
): Promise<boolean> {
  if (!crashReportsEnabled()) return false;
  try {
    await submitCrashReport(
      buildCrashReportPayload("error_boundary", error, { componentStack }),
    );
    return true;
  } catch (reportError) {
    console.error("Failed to submit crash report:", reportError);
    return false;
  }
}
