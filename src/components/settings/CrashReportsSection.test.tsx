import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import "../../i18n";
import { CrashReportsSection } from "./CrashReportsSection";
import type { CrashReportEntry } from "@/lib/crashReport";

const sampleReports: CrashReportEntry[] = [
  {
    id: "a",
    created_at: "2026-06-01T12:00:00.000Z",
    kind: "uncaught",
    message: "First crash",
    stack: null,
    component_stack: null,
    url: "/library",
    app_version: "1.0.4",
  },
  {
    id: "b",
    created_at: "2026-06-01T11:00:00.000Z",
    kind: "error_boundary",
    message: "Second crash",
    stack: "Error: boom",
    component_stack: "at App",
    url: "/game/test",
    app_version: "1.0.4",
  },
];

vi.mock("@/lib/api", () => ({
  listCrashReports: vi.fn(() => Promise.resolve(sampleReports)),
  clearCrashReports: vi.fn(() => Promise.resolve()),
}));

vi.mock("@/lib/crashReport", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@/lib/crashReport")>();
  return {
    ...actual,
    openCrashReportIssue: vi.fn(),
    copyCrashReport: vi.fn(() => Promise.resolve()),
  };
});

function renderSection() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <CrashReportsSection enabled onEnabledChange={() => {}} />
    </QueryClientProvider>,
  );
}

describe("CrashReportsSection", () => {
  it("lists all saved crash reports", async () => {
    renderSection();
    expect(await screen.findByText("First crash")).toBeInTheDocument();
    expect(screen.getByText("Second crash")).toBeInTheDocument();
    expect(screen.getAllByRole("button", { name: /copy report|скопировать отчёт/i })).toHaveLength(2);
  });
});
