import { render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { beforeEach, describe, expect, it } from "vitest";
import i18n from "@/i18n";
import { mockInvokeHandlers } from "@/test/mockTauri";
import { testGame } from "@/test/fixtures/gameProfile";
import type { InputDocument } from "@/lib/api/bindings";
import { InputEditor } from "./InputEditor";

const sourcePath = `${testGame.config_dir}\\Input.ini`;
const customSettingsPath = `${testGame.config_dir}\\GameUserSettings.ini`;
const renderEditor = () => {
  render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <InputEditor game={testGame} running={false} />
    </QueryClientProvider>,
  );
};
const show = (overrides: Partial<InputDocument> = {}) => {
  mockInvokeHandlers.get_input_document = (): InputDocument => ({
    revision: "test-revision",
    entries: [],
    source_path: sourcePath,
    file_state: "empty",
    custom_settings_path: null,
    ...overrides,
  });
  renderEditor();
};

describe("InputEditor source and coverage", () => {
  beforeEach(async () => {
    await i18n.changeLanguage("en");
  });

  it("explains an empty file and hides actions that cannot be used", async () => {
    show();
    expect(await screen.findByText(`File: ${sourcePath}`)).toBeVisible();
    expect(screen.getByText(/it is empty and contains no bindings/)).toBeVisible();
    expect(screen.queryByRole("button", { name: "Apply" })).not.toBeInTheDocument();
    expect(screen.queryByRole("combobox")).not.toBeInTheDocument();
  });

  it.each([
    ["missing", "Input.ini is missing from the active config folder."],
    ["no_classic_bindings", "Input.ini was found, but it contains no classic ActionMappings, AxisMappings or AxisConfig."],
  ] as const)("distinguishes %s", async (state, message) => {
    show({ file_state: state });
    expect(await screen.findByText(message)).toBeVisible();
  });

  it.each(["en", "ru"])("identifies PUBG custom settings in %s", async (language) => {
    await i18n.changeLanguage(language);
    show({ custom_settings_path: customSettingsPath });
    const hint = await screen.findByText(/CustomInputSettins/);
    expect(hint).toHaveTextContent(customSettingsPath);
    expect(hint).toHaveTextContent(language === "ru" ? "пока не поддерживает" : "does not yet support");
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
  });

  it("shows read errors without claiming that the file has no bindings", async () => {
    mockInvokeHandlers.get_input_document = () => {
      throw new Error("Access denied");
    };
    renderEditor();
    expect(await screen.findByRole("alert")).toHaveTextContent("Access denied");
    expect(screen.queryByText(/contains no bindings/)).not.toBeInTheDocument();
    expect(screen.queryByText(/Having an Input.ini/)).not.toBeInTheDocument();
  });
});
