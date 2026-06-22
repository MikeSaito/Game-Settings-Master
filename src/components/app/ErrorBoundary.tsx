import { Component, type ErrorInfo, type ReactNode } from "react";
import i18n from "../../i18n";

interface Props {
  children: ReactNode;
  /** Reset boundary when resetKey changes (e.g. different game/tab). */
  resetKey?: string;
}

interface State {
  error: Error | null;
}

/** Catches page render errors so one failure does not white-screen the whole app. */
export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("UI error boundary:", error, info.componentStack);
  }

  componentDidUpdate(prev: Props) {
    if (prev.resetKey !== this.props.resetKey && this.state.error) {
      this.setState({ error: null });
    }
  }

  render() {
    if (this.state.error) {
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
          <button
            type="button"
            onClick={() => this.setState({ error: null })}
            className="rounded-lg bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-white transition hover:opacity-90"
          >
            {i18n.t("common:errorBoundaryRetry")}
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}
