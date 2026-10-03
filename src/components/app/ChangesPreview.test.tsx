import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import i18n from "@/i18n";
import { mockInvoke, mockInvokeHandlers } from "@/test/mockTauri";
import { ChangesPreview } from "./ChangesPreview";
import { showPrepared } from "@/lib/api/preparedChanges";
import type { PreparedChanges, PrepareRequest } from "@/lib/api/bindings";

const request: PrepareRequest = {
  game_id: "test-game",
  config_dir: "C:\\Games\\Test\\Saved\\Config\\Windows",
};
const plan = (grouped = false): PreparedChanges => ({
  id: "preview-test",
  game_id: request.game_id,
  config_dir: request.config_dir,
  issues: [],
  revisions: {},
  operations: [
    {
      id: "a",
      group: "first",
      file: "Game.ini",
      section: "A",
      key: "x",
      before: null,
      after: "",
      kind: "scalar",
    },
    {
      id: "b",
      group: grouped ? "first" : "second",
      file: "Game.ini",
      section: "B",
      key: "y",
      before: "2",
      after: null,
      kind: "scalar",
    },
  ],
});
const open = async (preview: PreparedChanges) => {
  let result!: ReturnType<typeof showPrepared>;
  await act(async () => {
    result = showPrepared(preview, request);
  });
  return { result };
};

describe("prepared changes confirmation", () => {
  beforeEach(async () => {
    await i18n.changeLanguage("en");
    mockInvoke.mockClear();
    mockInvokeHandlers.validate_prepared_changes = () => [];
    mockInvokeHandlers.discard_prepared_changes = () => null;
    mockInvokeHandlers.apply_prepared_changes = () => ({
      backup_id: "snapshot",
      changed_files: ["Game.ini"],
      diff: [],
    });
  });
  it("cancels without committing and distinguishes absent from an empty string", async () => {
    render(<ChangesPreview />);
    const { result } = await open(plan());
    const cancelled = result.catch((error: Error) => error.message);
    expect(screen.getByText("Absent")).toBeVisible();
    expect(screen.getByText("Empty string")).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(await cancelled).toBe("Application cancelled");
    expect(
      mockInvoke.mock.calls.some(
        ([command]) => command === "apply_prepared_changes",
      ),
    ).toBe(false);
    expect(mockInvoke).toHaveBeenCalledWith("discard_prepared_changes", {
      planId: "preview-test",
    });
  });
  it("revalidates selection and commits only included operations", async () => {
    render(<ChangesPreview />);
    const { result } = await open(plan());
    fireEvent.click(screen.getByRole("checkbox", { name: "Game.ini: y" }));
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Confirm and apply" }),
      ).toBeEnabled(),
    );
    fireEvent.click(screen.getByRole("button", { name: "Confirm and apply" }));
    await act(async () => {
      await result;
    });
    expect(mockInvoke).toHaveBeenCalledWith("validate_prepared_changes", {
      planId: "preview-test",
      selectedIds: ["a"],
    });
    expect(mockInvoke).toHaveBeenCalledWith("apply_prepared_changes", {
      planId: "preview-test",
      selectedIds: ["a"],
      warningsAcknowledged: false,
    });
  });
  it("selects linked rows together and keeps a translated preview open", async () => {
    render(<ChangesPreview />);
    const { result } = await open(plan(true));
    const cancelled = result.catch(() => {});
    fireEvent.click(screen.getByRole("checkbox", { name: "Game.ini: x" }));
    expect(
      screen.getByRole("checkbox", { name: "Game.ini: y" }),
    ).not.toBeChecked();
    await act(async () => {
      await i18n.changeLanguage("ru");
    });
    expect(screen.getByRole("dialog")).toBeVisible();
    expect(
      screen.getByRole("button", {
        name: i18n.t("improvements:preview.confirm"),
      }),
    ).toBeDisabled();
    fireEvent.click(
      screen.getByRole("button", { name: i18n.t("improvements:cancel") }),
    );
    await cancelled;
  });
  it("requires a refreshed diff after a stale commit and uses the new server plan", async () => {
    render(<ChangesPreview />);
    let result!: ReturnType<typeof showPrepared>;
    const refreshed = { ...plan(), id: "refreshed" };
    const reprepare = vi.fn(async () => refreshed);
    mockInvokeHandlers.apply_prepared_changes = (args) => {
      if (args?.planId === "preview-test")
        throw new Error("Files changed: refresh the preview");
      return { backup_id: "new", changed_files: ["Game.ini"], diff: [] };
    };
    await act(async () => {
      result = showPrepared(plan(), request, reprepare);
    });
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Confirm and apply" }),
      ).toBeEnabled(),
    );
    fireEvent.click(screen.getByRole("button", { name: "Confirm and apply" }));
    await waitFor(() =>
      expect(screen.getByRole("alert")).toHaveTextContent("Files changed"),
    );
    expect(
      screen.getByRole("button", { name: "Confirm and apply" }),
    ).toBeDisabled();
    fireEvent.click(screen.getByRole("button", { name: "Refresh diff" }));
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Confirm and apply" }),
      ).toBeEnabled(),
    );
    fireEvent.click(screen.getByRole("button", { name: "Confirm and apply" }));
    await act(async () => {
      await result;
    });
    expect(reprepare).toHaveBeenCalledOnce();
    expect(mockInvoke).toHaveBeenCalledWith("discard_prepared_changes", {
      planId: "preview-test",
    });
    expect(mockInvoke).toHaveBeenCalledWith("apply_prepared_changes", {
      planId: "refreshed",
      selectedIds: ["a", "b"],
      warningsAcknowledged: false,
    });
  });
});
