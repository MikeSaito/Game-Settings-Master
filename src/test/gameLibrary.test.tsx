import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { AppWindowFocusProvider } from "../context/AppWindowFocusProvider";
import { GameLibrary } from "../pages/GameLibrary";
import { testGame } from "./fixtures/gameProfile";
import type { GameProfile } from "../lib/types";
import "../i18n";

function renderGameLibrary({
  games = [testGame],
  onSelectGame = vi.fn(),
}: {
  games?: GameProfile[];
  onSelectGame?: (game: GameProfile) => void;
} = {}) {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false, staleTime: Infinity, refetchOnMount: false },
    },
  });
  queryClient.setQueryData(["games"], games);
  const view = render(
    <QueryClientProvider client={queryClient}>
      <AppWindowFocusProvider>
        <GameLibrary selectedGame={null} onSelectGame={onSelectGame} />
      </AppWindowFocusProvider>
    </QueryClientProvider>,
  );
  return { ...view, onSelectGame };
}

describe("GameLibrary smoke", () => {
  it("renders library header", async () => {
    renderGameLibrary();
    expect(await screen.findByText("Game library")).toBeInTheDocument();
  });

  it("renders mocked game card name", async () => {
    renderGameLibrary();
    const nameEls = await screen.findAllByText(testGame.name);
    expect(nameEls.length).toBeGreaterThan(0);
  });

  it("renders Select action for a UE game card", async () => {
    renderGameLibrary();
    const selectButtons = await screen.findAllByRole("button", {
      name: "Select",
    });
    expect(selectButtons.length).toBeGreaterThan(0);
  });

  it("clicking Select calls onSelectGame with the game", async () => {
    const user = userEvent.setup();
    const { onSelectGame } = renderGameLibrary();

    await user.click(await screen.findByRole("button", { name: "Select" }));

    expect(onSelectGame).toHaveBeenCalledWith(testGame);
  });

  it("shows Set config as the primary action when config_dir is missing", async () => {
    const user = userEvent.setup();
    const gameWithoutConfig = { ...testGame, config_dir: null };
    const { onSelectGame } = renderGameLibrary({ games: [gameWithoutConfig] });

    const configButtons = await screen.findAllByRole("button", { name: "Set config" });
    expect(configButtons.length).toBeGreaterThan(0);

    await user.click(configButtons[0]);

    expect(onSelectGame).not.toHaveBeenCalled();
  });

  it("keeps card actions visible in list view", async () => {
    const user = userEvent.setup();
    renderGameLibrary();

    await user.click(await screen.findByLabelText("List"));

    expect(await screen.findByRole("button", { name: "Select" })).toBeVisible();
    expect(screen.getByRole("button", { name: "Cover" })).toBeVisible();
  });
});
