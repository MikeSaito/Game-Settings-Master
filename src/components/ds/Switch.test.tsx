import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { Switch } from "./Switch";

describe("Switch", () => {
  it("calls onChange when clicked", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();

    render(<Switch checked={false} onChange={onChange} />);

    await user.click(screen.getByRole("switch"));

    expect(onChange).toHaveBeenCalledWith(true);
  });

  it("merges onClick without losing toggle behavior", async () => {
    const user = userEvent.setup();
    const onClick = vi.fn();
    const onChange = vi.fn();

    render(<Switch checked onClick={onClick} onChange={onChange} />);

    await user.click(screen.getByRole("switch"));

    expect(onClick).toHaveBeenCalled();
    expect(onChange).toHaveBeenCalledWith(false);
  });

  it("does not toggle when onClick prevents default", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();

    render(
      <Switch
        checked={false}
        onClick={(event) => event.preventDefault()}
        onChange={onChange}
      />,
    );

    await user.click(screen.getByRole("switch"));

    expect(onChange).not.toHaveBeenCalled();
  });
});
