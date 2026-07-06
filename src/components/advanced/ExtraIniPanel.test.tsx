import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import { ExtraIniPanel } from "./ExtraIniPanel";
import type { GameConfig } from "@/lib/core";
import "../../i18n";

const testConfigDir = "C:\\Games\\Test\\Saved\\Config\\Windows";

const sampleConfig: GameConfig = {
  config_dir: testConfigDir,
  files: {
    "Input.ini": {
      sections: {
        "/Script/Engine.InputSettings": {
          bEnableMouseSmoothing: "False",
        },
      },
    },
    "DeviceProfiles.ini": {
      sections: {
        DeviceProfileManager: {
          DeviceProfileSelection: "Windows",
        },
      },
    },
  },
};

describe("ExtraIniPanel", () => {
  it("shows loading state", () => {
    render(<ExtraIniPanel gameConfig={undefined} loading />);
    expect(screen.getByText("Loading ini…")).toBeInTheDocument();
  });

  it("shows empty state when no extra ini files", () => {
    render(<ExtraIniPanel gameConfig={{ config_dir: testConfigDir, files: {} }} />);
    expect(screen.getByText("No Input.ini / DeviceProfiles.ini")).toBeInTheDocument();
  });

  it("lists ini rows and filters by search", async () => {
    const user = userEvent.setup();
    render(<ExtraIniPanel gameConfig={sampleConfig} />);
    expect(screen.getByText("bEnableMouseSmoothing")).toBeInTheDocument();
    expect(screen.getByText("DeviceProfileSelection")).toBeInTheDocument();

    await user.type(screen.getByRole("textbox"), "mouse");
    expect(screen.queryByText("DeviceProfileSelection")).not.toBeInTheDocument();
    expect(screen.getByText("bEnableMouseSmoothing")).toBeInTheDocument();
  });
});
