import { Component, type ErrorInfo, type ReactNode } from "react";
import i18n from "@/i18n";
import { reportBoundaryError } from "@/hooks/app/useCrashReporting";
import { loadAppSettings } from "@/lib/settings";
import { copyCrashReport, openCrashReportIssue } from "@/lib/crashReport";
import { buildCrashReportPayload } from "@/lib/crashReport";

interface Props {
  children: ReactNode;
  /** Reset boundary when resetKey changes (e.g. different game/tab). */
  resetKey?: string;
}

interface State {
  error: Error | null;
  reportSaved: boolean;
  componentStack: string | null;
}

/** Catches page render errors so one failure does not white-screen the whole app. */
export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null, reportSaved: false, componentStack: null };

  static getDerivedStateFromError(error: Error): Partial<State> {
    return { error, reportSaved: false, componentStack: null };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    const componentStack = info.componentStack ?? null;
    console.error("UI error boundary:", error, componentStack);
    this.setState({ componentStack });
    void (async () => {
      const saved = await reportBoundaryError(error, componentStack);
      if (saved) this.setState({ reportSaved: true });
    })();
  }

  componentDidUpdate(prev: Props) {
    if (prev.resetKey !== this.props.resetKey && this.state.error) {
      this.setState({ error: null, reportSaved: false, componentStack: null });
    }
  }

  render() {
    if (this.state.error) {
      const crashReportsEnabled = loadAppSettings().crashReportsEnabled;
      const reportEntry = crashReportsEnabled
        ? {
            ...buildCrashReportPayload("error_boundary", this.state.error, {
              componentStack: this.state.componentStack,
            }),
            id: "local",
            created_at: new Date().toISOString(),
          }
        : null;

      return (
        <div className="flex flex-col items-center justify-center gap-4 py-20 text-center">
          <div className="text-lg font-semibold text-[var(--color-text)]">
            {i18n.t("common:errorBoundaryTitle")}
          </div>
          <p className="max-w-md text-sm text-muted">
            {i18n.t("common:errorBoundaryBody")}
          </p>
          <pre className="max-w-lg overflow-auto rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-hover)] p-3 text-left font-mono text-xs text-muted">
            {this.state.error.message}
          </pre>
          {crashReportsEnabled && this.state.reportSaved && (
            <p className="text-xs text-[var(--color-text-muted)]">
              {i18n.t("common:crashReportSaved")}
            </p>
          )}
          <div className="flex flex-wrap items-center justify-center gap-2">
            {reportEntry && (
              <>
                <button
                  type="button"
                  onClick={() => openCrashReportIssue(reportEntry)}
                  className="rounded-lg border border-[var(--color-border-strong)] bg-[var(--color-bg-soft)] px-4 py-2 text-sm font-medium text-[var(--color-text)] transition hover:bg-[var(--color-surface-hover)]"
                >
                  {i18n.t("common:crashReportSendGithub")}
                </button>
                <button
                  type="button"
                  onClick={() => void copyCrashReport(reportEntry)}
                  className="rounded-lg border border-[var(--color-border-strong)] bg-[var(--color-bg-soft)] px-4 py-2 text-sm font-medium text-[var(--color-text)] transition hover:bg-[var(--color-surface-hover)]"
                >
                  {i18n.t("common:crashReportCopy")}
                </button>
              </>
            )}
            <button
              type="button"
              onClick={() =>
                this.setState({ error: null, reportSaved: false, componentStack: null })
              }
              className="rounded-lg bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-white transition hover:opacity-90"
            >
              {i18n.t("common:errorBoundaryRetry")}
            </button>
          </div>
        </div>
      );
    }
    return this.props.children;
  }
}
