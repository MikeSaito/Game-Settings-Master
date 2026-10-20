import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { beforeEach, describe, expect, it, vi } from "vitest";
import i18n from "@/i18n";
import { mockInvokeHandlers } from "@/test/mockTauri";
import { testGame } from "@/test/fixtures/gameProfile";
import type { IniTextDocument } from "@/lib/api/bindings";
import * as prepared from "@/lib/api/preparedChanges";
import { IniTextEditor } from "./IniTextEditor";

const document = (
  file = "Engine.ini",
  text = "[X]\r\nKey=1\r\n",
  revision: string | null = "original",
): IniTextDocument => ({
  file,
  text,
  revision,
  config_dir: testGame.config_dir!,
  encoding: "UTF-8 BOM",
});
function show(running = false) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  const view = render(
    <QueryClientProvider client={client}>
      <IniTextEditor game={testGame} running={running} active />
    </QueryClientProvider>,
  );
  return { client, ...view };
}
describe("INI text editor", () => {
  beforeEach(async () => {
    vi.restoreAllMocks();
    await i18n.changeLanguage("en");
    mockInvokeHandlers.get_ini_text_document = (args) =>
      document(String(args?.file));
  });
  it("keeps dirty source text when a refetch discovers an external modification", async () => {
    const user = userEvent.setup();
    const { client } = show();
    const area = await screen.findByRole("textbox", { name: "INI file text" });
    await waitFor(() => expect(area).toHaveValue("[X]\nKey=1\n"));
    fireEvent.change(area, { target: { value: "[X]\nKey=draft\n" } });
    act(() =>
      client.setQueryData(
        ["ini-text", testGame.id, testGame.config_dir, "Engine.ini"],
        document("Engine.ini", "; external", "external"),
      ),
    );
    expect(await screen.findByText(/The file changed on disk/)).toBeVisible();
    expect(area).toHaveValue("[X]\nKey=draft\n");
    mockInvokeHandlers.get_ini_text_document = () =>
      document("Engine.ini", "; external", "external");
    await user.click(screen.getByRole("button", { name: "Reload file" }));
    expect(area).toHaveValue("[X]\nKey=draft\n");
    await user.click(
      screen.getByRole("button", { name: "Discard draft and reload" }),
    );
    await waitFor(() => expect(area).toHaveValue("; external"));
    expect(
      screen.getByRole("button", { name: "Preview and save" }),
    ).toBeDisabled();
  });
  it("preserves CRLF in requests and retains files excluded from the preview", async () => {
    const user = userEvent.setup();
    const save = vi
      .spyOn(prepared, "previewAndApply")
      .mockImplementation(async (request) => {
        const raw = request.raw_files!;
        expect(raw).toHaveLength(2);
        expect(raw[0].text).toBe("[X]\r\nKey=engine\r\n");
        expect(raw[0].revision).toBe("original");
        mockInvokeHandlers.get_ini_text_document = (args) =>
          document(
            String(args?.file),
            raw.find((item) => item.file === args?.file)?.text ?? "",
          );
        return {
          backup_id: "snapshot",
          changed_files: ["Engine.ini"],
          diff: [],
        };
      });
    show();
    const area = screen.getByRole("textbox", { name: "INI file text" });
    await waitFor(() =>
      expect((area as HTMLTextAreaElement).value).toContain("Key=1"),
    );
    fireEvent.change(area, { target: { value: "[X]\nKey=engine\n" } });
    await user.selectOptions(screen.getByRole("combobox"), "Input.ini");
    await waitFor(() =>
      expect((area as HTMLTextAreaElement).value).toContain("Key=1"),
    );
    fireEvent.change(area, { target: { value: "[X]\nKey=input\n" } });
    await user.click(screen.getByRole("button", { name: "Preview and save" }));
    await waitFor(() => expect(save).toHaveBeenCalledOnce());
    await waitFor(() =>
      expect(screen.getByText("Modified files: 1")).toBeVisible(),
    );
    expect(area).toHaveValue("[X]\nKey=input\n");
  });
  it("keeps the draft after a failed preview and blocks saving while the game runs", async () => {
    const user = userEvent.setup();
    vi.spyOn(prepared, "previewAndApply").mockRejectedValue(
      new Error("File changed: reload it before saving"),
    );
    const { rerender } = show();
    const area = screen.getByRole("textbox", { name: "INI file text" });
    await waitFor(() =>
      expect((area as HTMLTextAreaElement).value).toContain("Key=1"),
    );
    fireEvent.change(area, { target: { value: "draft" } });
    await user.click(screen.getByRole("button", { name: "Preview and save" }));
    expect(
      await screen.findByText("File changed: reload it before saving"),
    ).toBeVisible();
    expect(area).toHaveValue("draft");
    rerender(
      <QueryClientProvider client={new QueryClient()}>
        <IniTextEditor game={testGame} running active />
      </QueryClientProvider>,
    );
    expect(
      screen.getByRole("button", { name: "Preview and save" }),
    ).toBeDisabled();
  });
  it("reports a completed write even if rereading the saved file fails", async () => {
    const user = userEvent.setup();
    vi.spyOn(prepared, "previewAndApply").mockImplementation(async () => {
      mockInvokeHandlers.get_ini_text_document = () => {
        throw new Error("Read access denied");
      };
      return { backup_id: "snapshot", changed_files: ["Engine.ini"], diff: [] };
    });
    const { client } = show();
    const invalidate = vi.spyOn(client, "invalidateQueries");
    const area = screen.getByRole("textbox", { name: "INI file text" });
    await waitFor(() =>
      expect((area as HTMLTextAreaElement).value).toContain("Key=1"),
    );
    fireEvent.change(area, { target: { value: "; saved draft" } });
    await user.click(screen.getByRole("button", { name: "Preview and save" }));
    expect(
      await screen.findByText("Files saved: 1. Backup created."),
    ).toBeVisible();
    expect(screen.getByText("Engine.ini: Read access denied")).toBeVisible();
    expect(invalidate).toHaveBeenCalledWith({
      queryKey: ["parameters", testGame.config_dir, testGame.id],
    });
    expect(area).toHaveValue("; saved draft");
  });
  it("reports read failures without presenting an empty editable file", async () => {
    mockInvokeHandlers.get_ini_text_document = () => {
      throw new Error("Unsupported encoding");
    };
    show();
    expect(await screen.findByText("Unsupported encoding")).toBeVisible();
    expect(
      screen.getByRole("textbox", { name: "INI file text" }),
    ).toHaveAttribute("readonly");
    expect(
      screen.getByRole("button", { name: "Preview and save" }),
    ).toBeDisabled();
  });
});
