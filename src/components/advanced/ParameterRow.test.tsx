import { render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ParameterRow } from "./ParameterRow";
import type { GameParameter } from "../../lib/types";
import "../../i18n";

function baseParam(overrides: Partial<GameParameter>): GameParameter {
  return {
    key: "sg.ShadowQuality",
    section: "ScalabilityGroups",
    file: "GameUserSettings.ini",
    value: "2",
    title: "Shadow quality",
    description: "Shadow preset index.",
    impact: "Higher = better shadows.",
    category: "Scalability",
    min: "0",
    max: "4",
    in_game_label: null,
    value_hint: null,
    value_type: "int",
    known: true,
    editable: true,
    present_in_ini: true,
    default_value: null,
    ui_control: "slider",
    step: "1",
    options: null,
    recommended: null,
    catalog_recommended: true,
    tier_hint: null,
    ...overrides,
  };
}

describe("ParameterRow", () => {
  it("renders compact row metadata and tier hint", () => {
    render(
      <ParameterRow
        param={baseParam({
          tier_hint: "Low (0): r.ShadowQuality=0 · Medium (1): r.ShadowQuality=1",
        })}
      />,
    );

    expect(screen.getByTestId("parameter-row")).toBeInTheDocument();
    expect(screen.getByText("Shadow quality")).toBeInTheDocument();
    expect(screen.getByText("High")).toBeInTheDocument();
  });

  it("renders select control options", () => {
    render(
      <ParameterRow
        param={baseParam({
          key: "r.AntiAliasingMethod",
          value: "2",
          ui_control: "select",
          options: [
            { value: "0", label: "None" },
            { value: "2", label: "TAA" },
          ],
        })}
        editable
        onChange={vi.fn()}
      />,
    );

    expect(screen.getByRole("combobox")).toBeInTheDocument();
  });

  it("shows resolution quality as percent", () => {
    render(
      <ParameterRow
        param={baseParam({
          key: "sg.ResolutionQuality",
          value: "75",
          value_type: "float",
          min: "25",
          max: "200",
        })}
        editable
        onChange={vi.fn()}
      />,
    );

    expect(screen.getByText("75%")).toBeInTheDocument();
  });

  it("uses human labels for fullscreen mode", () => {
    render(
      <ParameterRow
        param={baseParam({
          key: "FullscreenMode",
          section: "/Script/Engine.GameUserSettings",
          category: "Display",
          value: "1",
          value_type: "int",
          min: "0",
          max: "2",
          ui_control: "select",
          options: null,
        })}
        editable
        onChange={vi.fn()}
      />,
    );

    expect(screen.getByRole("option", { name: "Borderless" })).toBeInTheDocument();
  });

  it("renders humanized title when title equals raw key", () => {
    render(
      <ParameterRow
        param={baseParam({
          key: "r.ViewDistanceScale",
          title: "r.ViewDistanceScale",
        })}
      />,
    );

    const row = screen.getByTestId("parameter-row");
    expect(within(row).getByText(/view · distance · scale/i)).toBeInTheDocument();
  });

  it("renders conflict chip", () => {
    render(
      <ParameterRow
        param={baseParam({ key: "sg.ShadowQuality" })}
        conflictLabel="sg/r conflict"
      />,
    );

    expect(screen.getByText("sg/r conflict")).toBeInTheDocument();
  });
});
