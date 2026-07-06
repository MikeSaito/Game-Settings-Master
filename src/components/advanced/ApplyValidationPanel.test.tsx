import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { ApplyValidationPanel } from "./ApplyValidationPanel";
import type { ValidationIssue } from "@/lib/editor/validation";
import "../../i18n";

describe("ApplyValidationPanel", () => {
  it("renders nothing when there are no issues", () => {
    const { container } = render(
      <ApplyValidationPanel
        issues={[]}
        warningsAcknowledged={false}
        onWarningsAcknowledgedChange={vi.fn()}
      />,
    );
    expect(container).toBeEmptyDOMElement();
  });

  it("shows blocking errors", () => {
    const issues: ValidationIssue[] = [
      {
        code: "shipped_key_removal",
        severity: "error",
        key: "sg.ShadowQuality",
        i18nKey: "validation.shippedKeyRemoval",
        i18nParams: { key: "sg.ShadowQuality" },
      },
    ];
    render(
      <ApplyValidationPanel
        issues={issues}
        warningsAcknowledged={false}
        onWarningsAcknowledgedChange={vi.fn()}
      />,
    );
    expect(screen.getByText("Fix before apply")).toBeInTheDocument();
  });

  it("shows warnings with acknowledge checkbox when no errors", async () => {
    const user = userEvent.setup();
    const onAck = vi.fn();
    const issues: ValidationIssue[] = [
      {
        code: "value_out_of_range",
        severity: "warning",
        key: "sg.ShadowQuality",
        i18nKey: "validation.valueAboveMax",
        i18nParams: { key: "sg.ShadowQuality", value: "9", max: "4" },
      },
    ];
    render(
      <ApplyValidationPanel
        issues={issues}
        warningsAcknowledged={false}
        onWarningsAcknowledgedChange={onAck}
      />,
    );
    expect(screen.getByText("Warnings")).toBeInTheDocument();
    await user.click(screen.getByRole("checkbox"));
    expect(onAck).toHaveBeenCalledWith(true);
  });
});
