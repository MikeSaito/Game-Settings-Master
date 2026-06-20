import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { AppWindowFocusProvider } from "../context/AppWindowFocusProvider";
import { GameLibrary } from "../pages/GameLibrary";
import { testGame } from "./fixtures/gameProfile";
import "../i18n";

function renderGameLibrary() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false, staleTime: Infinity, refetchOnMount: false },
    },
  });
  queryClient.setQueryData(["games"], [testGame]);
  return render(
    <QueryClientProvider client={queryClient}>
      <AppWindowFocusProvider>
        <GameLibrary selectedGame={null} onSelectGame={vi.fn()} />
      </AppWindowFocusProvider>
    </QueryClientProvider>,
  );
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
});
