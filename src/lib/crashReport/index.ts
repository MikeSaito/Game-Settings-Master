export type CrashReportKind = "error_boundary" | "uncaught" | "unhandled_rejection";

export interface CrashReportPayload {
  kind: CrashReportKind;
  message: string;
  stack?: string | null;
  component_stack?: string | null;
  url?: string | null;
  app_version: string;
}

export interface CrashReportEntry extends CrashReportPayload {
  id: string;
  created_at: string;
}

const GITHUB_REPO = "MikeSaito/Game-Settings-Master";

export function buildCrashReportPayload(
  kind: CrashReportKind,
  error: unknown,
  extras?: { componentStack?: string | null; url?: string | null },
): CrashReportPayload {
  const message = error instanceof Error ? error.message : String(error);
  const stack = error instanceof Error ? error.stack ?? null : null;
  return {
    kind,
    message,
    stack,
    component_stack: extras?.componentStack ?? null,
    url: extras?.url ?? (typeof window !== "undefined" ? window.location.pathname : null),
    app_version: __APP_VERSION__,
  };
}

export function formatCrashReportBody(entry: CrashReportEntry): string {
  const lines = [
    "## Crash report",
    "",
    `- **App version:** ${entry.app_version}`,
    `- **Kind:** ${entry.kind}`,
    `- **URL:** ${entry.url ?? "n/a"}`,
    `- **Time:** ${entry.created_at}`,
    "",
    "### Message",
    "```",
    entry.message,
    "```",
  ];
  if (entry.stack) {
    lines.push("", "### Stack", "```", entry.stack, "```");
  }
  if (entry.component_stack) {
    lines.push("", "### React component stack", "```", entry.component_stack, "```");
  }
  lines.push(
    "",
    "---",
    "_Submitted via Game Settings Master opt-in crash reporting (no behavior analytics)._",
  );
  return lines.join("\n");
}

export function githubIssueUrlForReport(entry: CrashReportEntry): string {
  const title = encodeURIComponent(`[Crash] ${entry.message.slice(0, 80)}`);
  const body = encodeURIComponent(formatCrashReportBody(entry));
  return `https://github.com/${GITHUB_REPO}/issues/new?title=${title}&body=${body}`;
}

export async function copyCrashReport(entry: CrashReportEntry): Promise<void> {
  await navigator.clipboard.writeText(formatCrashReportBody(entry));
}

export function openCrashReportIssue(entry: CrashReportEntry): void {
  window.open(githubIssueUrlForReport(entry), "_blank", "noopener,noreferrer");
}
