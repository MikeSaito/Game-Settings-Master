import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import { useEffect } from "react";
import { MemoryRouter, Navigate, Route, Routes, useLocation, useNavigate } from "react-router-dom";
import { describe, expect, it, vi } from "vitest";
import { AppWindowFocusProvider } from "@/context/AppWindowFocusProvider";
import { EditorModeBar } from "@/components/advanced/EditorModeBar";
import { GameLibrary } from "@/pages/GameLibrary";
import { libraryPath, parseGameRoute, parseLegacyGameRoute } from "@/lib/routing";
import { LegacyGameRouteRedirect } from "@/lib/routing";
import { testGame } from "./fixtures/gameProfile";
import "../i18n";

function LocationProbe({ onPath }: { onPath: (path: string) => void }) {
  const { pathname } = useLocation();
  useEffect(() => {
    onPath(pathname);
  }, [pathname, onPath]);
  return null;
}

function TestRoutes({
  games,
  onPath,
}: {
  games: typeof testGame[];
  onPath: (path: string) => void;
}) {
  const location = useLocation();
  const navigate = useNavigate();
  const gameRoute = parseGameRoute(location.pathname);
  const selectedGame = gameRoute
    ? games.find((g) => g.id === gameRoute.gameId) ?? null
    : null;

  useEffect(() => {
    if (gameRoute && games.length > 0 && !selectedGame) {
      navigate(libraryPath(), { replace: true });
    }
  }, [gameRoute, games.length, selectedGame, navigate]);

  return (
    <>
      <LocationProbe onPath={onPath} />
      <Routes>
        <Route path="/" element={<Navigate to={libraryPath()} replace />} />
        <Route
          path="/library"
          element={
            <GameLibrary
              selectedGame={selectedGame}
              onSelectGame={() => {}}
            />
          }
        />
        <Route
          path="/game/:gameId/wizard"
          element={<LegacyGameRouteRedirect games={games} />}
        />
        <Route
          path="/game/:gameId/backups"
          element={<LegacyGameRouteRedirect games={games} />}
        />
        <Route path="*" element={<Navigate to={libraryPath()} replace />} />
      </Routes>
    </>
  );
}

describe("router integration", () => {
  it("renders GameLibrary on /library", async () => {
    const queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false, staleTime: Infinity, refetchOnMount: false },
      },
    });
    queryClient.setQueryData(["games"], [testGame]);
    render(
      <QueryClientProvider client={queryClient}>
        <AppWindowFocusProvider>
          <MemoryRouter initialEntries={["/library"]}>
            <TestRoutes games={[testGame]} onPath={() => {}} />
          </MemoryRouter>
        </AppWindowFocusProvider>
      </QueryClientProvider>,
    );
    expect(await screen.findByText("Game library")).toBeInTheDocument();
  });

  it("treats removed wizard and reshade routes as unknown to parseGameRoute", () => {
    expect(parseGameRoute("/game/foo/wizard")).toBeNull();
    expect(parseGameRoute("/game/foo/reshade")).toBeNull();
    expect(parseLegacyGameRoute("/game/foo/wizard")).toEqual({ gameId: "foo" });
    expect(parseGameRoute("/game/foo/advanced")).toEqual({
      gameId: "foo",
      tab: "advanced",
    });
  });

  it("redirects legacy wizard URL to advanced for known game", async () => {
    const paths: string[] = [];
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    queryClient.setQueryData(["games"], [testGame]);
    render(
      <QueryClientProvider client={queryClient}>
        <AppWindowFocusProvider>
          <MemoryRouter initialEntries={[`/game/${testGame.id}/wizard`]}>
            <TestRoutes
              games={[testGame]}
              onPath={(p) => paths.push(p)}
            />
          </MemoryRouter>
        </AppWindowFocusProvider>
      </QueryClientProvider>,
    );
    await waitFor(() => {
      expect(paths.some((p) => p.includes("/advanced"))).toBe(true);
    });
  });

  it("redirects legacy backups URL to advanced and stores panel", async () => {
    const paths: string[] = [];
    const setItem = vi.spyOn(Storage.prototype, "setItem");
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    queryClient.setQueryData(["games"], [testGame]);
    render(
      <QueryClientProvider client={queryClient}>
        <AppWindowFocusProvider>
          <MemoryRouter initialEntries={[`/game/${testGame.id}/backups`]}>
            <TestRoutes games={[testGame]} onPath={(p) => paths.push(p)} />
          </MemoryRouter>
        </AppWindowFocusProvider>
      </QueryClientProvider>,
    );
    await waitFor(() => {
      expect(paths.some((p) => p.includes("/advanced"))).toBe(true);
    });
    expect(setItem).toHaveBeenCalledWith(`gsm-editor-panel:${testGame.id}`, "backups");
    setItem.mockRestore();
  });

  it("redirects unknown game id to /library", async () => {
    const paths: string[] = [];
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    queryClient.setQueryData(["games"], [testGame]);
    render(
      <QueryClientProvider client={queryClient}>
        <AppWindowFocusProvider>
          <MemoryRouter initialEntries={["/game/missing-id/advanced"]}>
            <TestRoutes
              games={[testGame]}
              onPath={(p) => paths.push(p)}
            />
          </MemoryRouter>
        </AppWindowFocusProvider>
      </QueryClientProvider>,
    );
    await waitFor(() => {
      expect(paths).toContain("/library");
    });
  });

  it("renders backups tab in editor mode bar", () => {
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    render(
      <QueryClientProvider client={queryClient}>
        <AppWindowFocusProvider>
          <MemoryRouter initialEntries={[`/game/${testGame.id}/advanced`]}>
            <EditorModeBar
              gameId={testGame.id}
              panel="basic"
              onPanelChange={() => {}}
              engineStats={{ total: 0, on: 0, off: 0 }}
            />
          </MemoryRouter>
        </AppWindowFocusProvider>
      </QueryClientProvider>,
    );

    expect(screen.getByRole("tab", { name: /backups|бекапы/i })).toBeInTheDocument();
  });

});
