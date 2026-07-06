import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import { useEffect } from "react";
import { MemoryRouter, Route, Routes, useLocation } from "react-router-dom";
import { describe, expect, it, vi } from "vitest";
import { GameEditorPage } from "@/App";
import { libraryPath } from "@/lib/routing";
import { testGame } from "@/test/fixtures/gameProfile";
import "../i18n";

function LocationProbe({ onPath }: { onPath: (path: string) => void }) {
  const { pathname } = useLocation();
  useEffect(() => {
    onPath(pathname);
  }, [pathname, onPath]);
  return null;
}

describe("GameEditorPage", () => {
  it("redirects to library when game id is unknown", async () => {
    const onPath = vi.fn();
    const queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false, staleTime: Infinity, refetchOnMount: false },
      },
    });
    queryClient.setQueryData(["games"], [testGame]);

    render(
      <QueryClientProvider client={queryClient}>
        <MemoryRouter initialEntries={["/game/missing-game/advanced"]}>
          <LocationProbe onPath={onPath} />
          <Routes>
            <Route
              path="/game/:gameId/advanced"
              element={
                <GameEditorPage games={[testGame]} gamesLoading={false} />
              }
            />
            <Route path="/library" element={<div>Library</div>} />
          </Routes>
        </MemoryRouter>
      </QueryClientProvider>,
    );

    await waitFor(() => {
      expect(onPath).toHaveBeenCalledWith(libraryPath());
    });
  });

  it("shows not-found state while games list is still empty", () => {
    render(
      <MemoryRouter initialEntries={["/game/missing-game/advanced"]}>
        <Routes>
          <Route
            path="/game/:gameId/advanced"
            element={<GameEditorPage games={[]} gamesLoading={false} />}
          />
        </Routes>
      </MemoryRouter>,
    );
    expect(screen.getByText("Game not found")).toBeInTheDocument();
  });
});
