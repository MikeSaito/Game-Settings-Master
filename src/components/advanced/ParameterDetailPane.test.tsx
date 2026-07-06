import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { ParameterDetailPane } from "./ParameterDetailPane";
import type { GameParameter } from "@/lib/core";
import "../../i18n";

function baseParam(overrides: Partial<GameParameter>): GameParameter {
  return {
    key: "sg.ShadowQuality",
    section: "ScalabilityGroups",
    file: "GameUserSettings.ini",
    value: "2",
    title: "Shadow quality",
    description: "Shadow preset description.",
    impact: "Higher means better shadows.",
    category: "Scalability",
    min: "0",
    max: "4",
    in_game_label: null,
    value_hint: "0 Low → 4 max",
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

describe("ParameterDetailPane", () => {
  it("shows hover hint when no parameter selected", () => {
    render(<ParameterDetailPane param={null} />);
    expect(screen.getByTestId("parameter-list-details")).toBeInTheDocument();
    expect(
      screen.getByText(/Hover a parameter in the list on the left/i),
    ).toBeInTheDocument();
  });

  it("renders parameter description and impact", () => {
    render(
      <ParameterDetailPane
        param={baseParam({
          description: "Detailed shadow preset info.",
          impact: "FPS drops on high settings.",
          tier_hint: "Low (0): r.ShadowQuality=0",
        })}
      />,
    );
    expect(screen.getByText("Detailed shadow preset info.")).toBeInTheDocument();
    expect(screen.getByText(/FPS drops on high settings/)).toBeInTheDocument();
    expect(screen.getByText(/r.ShadowQuality=0/)).toBeInTheDocument();
  });
});
