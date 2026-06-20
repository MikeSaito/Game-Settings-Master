import { MemoryRouter, useLocation } from "react-router-dom";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useEffect } from "react";
import { describe, expect, it, vi } from "vitest";
import "../../i18n";
import { NavRail } from "./NavRail";
import { testGame } from "../../test/fixtures/gameProfile";

function LocationProbe({ onPath }: { onPath: (path: string) => void }) {
  const { pathname } = useLocation();
  useEffect(() => {
    onPath(pathname);
  }, [pathname, onPath]);
  return null;
}

describe("NavRail", () => {
  it("renders settings button", () => {
    render(
      <MemoryRouter>
        <NavRail active="library" selectedGame={null} onSettingsClick={vi.fn()} />
      </MemoryRouter>,
    );
    expect(screen.getByRole("button", { name: /settings|настройки/i })).toBeInTheDocument();
  });

  it("navigates to library from a game tab", async () => {
    const user = userEvent.setup();
    const paths: string[] = [];

    render(
      <MemoryRouter initialEntries={[`/game/${testGame.id}/advanced`]}>
        <LocationProbe onPath={(path) => paths.push(path)} />
        <NavRail active="advanced" selectedGame={testGame} onSettingsClick={vi.fn()} />
      </MemoryRouter>,
    );

    await user.click(screen.getByRole("button", { name: /library|библиотека/i }));

    await waitFor(() => {
      expect(paths).toContain("/library");
    });
  });
});
