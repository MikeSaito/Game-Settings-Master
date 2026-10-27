import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import i18n from "@/i18n";
import { UpdateGate } from "./UpdateGate";

vi.unmock("@/components/app/UpdateGate");
const state = vi.hoisted(() => ({
  status: "checking",
  update: null as null | { version: string },
  error: null as string | null,
  progress: null as null | { downloaded: number; total: number },
  retry: vi.fn(),
  installUpdate: vi.fn(),
  continueWithoutUpdate: vi.fn(),
  canBypassOnError: false,
}));
vi.mock("@/hooks/app/useAppUpdater", () => ({ useAppUpdater: () => state }));

beforeEach(async () => {
  Object.assign(state, {
    status: "checking",
    update: null,
    error: null,
    progress: null,
    canBypassOnError: false,
  });
  vi.clearAllMocks();
  await i18n.changeLanguage("en");
});

describe("startup update gate", () => {
  it("shows an indeterminate, accessible checking stage without blocking on fonts", () => {
    render(<UpdateGate>Library</UpdateGate>);
    expect(screen.getByRole("status")).toHaveTextContent(
      "Checking for updates",
    );
    expect(screen.getByRole("progressbar")).not.toHaveAttribute(
      "aria-valuenow",
    );
    expect(screen.queryByText("Library")).not.toBeInTheDocument();
  });
  it("keeps download progress indeterminate when the size is unknown", () => {
    Object.assign(state, {
      status: "downloading",
      progress: { downloaded: 1024, total: 0 },
    });
    render(<UpdateGate>Library</UpdateGate>);
    expect(screen.getByRole("progressbar")).not.toHaveAttribute(
      "aria-valuenow",
    );
    expect(screen.queryByText("0%")).not.toBeInTheDocument();
  });
  it("uses actual download progress and keeps installation visible afterward", () => {
    Object.assign(state, {
      status: "downloading",
      progress: { downloaded: 25, total: 100 },
    });
    const view = render(<UpdateGate>Library</UpdateGate>);
    expect(screen.getByRole("progressbar")).toHaveAttribute(
      "aria-valuenow",
      "25",
    );
    state.status = "installing";
    view.rerender(<UpdateGate>Library</UpdateGate>);
    expect(screen.getByRole("status")).toHaveTextContent(
      "Verifying and installing",
    );
    expect(screen.getByRole("progressbar")).not.toHaveAttribute(
      "aria-valuenow",
    );
  });
  it("allows the existing offline recovery only when the updater permits it", async () => {
    Object.assign(state, {
      status: "error",
      error: "Network unavailable",
      canBypassOnError: true,
    });
    const view = render(<UpdateGate>Library</UpdateGate>);
    await userEvent.click(
      screen.getByRole("button", { name: /Continue without updating/ }),
    );
    expect(state.continueWithoutUpdate).toHaveBeenCalledOnce();
    state.canBypassOnError = false;
    view.rerender(<UpdateGate>Library</UpdateGate>);
    expect(
      screen.queryByRole("button", { name: /Continue without updating/ }),
    ).not.toBeInTheDocument();
  });
  it("renders children only once startup update checking is ready", () => {
    state.status = "ready";
    render(<UpdateGate>Library</UpdateGate>);
    expect(screen.getByText("Library")).toBeInTheDocument();
    expect(screen.queryByTestId("startup-screen")).not.toBeInTheDocument();
  });
});
