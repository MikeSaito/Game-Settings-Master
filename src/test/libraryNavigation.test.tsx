import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useEffect } from "react";
import { MemoryRouter, Route, Routes, useLocation, useNavigate } from "react-router-dom";
import { describe, expect, it, vi } from "vitest";
import { AppShell } from "@/components/layout/AppShell";
import { AppWindowFocusProvider } from "@/context/AppWindowFocusProvider";
import { GameLibrary } from "@/pages/GameLibrary";
import { libraryPath, gameTabPath } from "@/lib/routing";
import { testGame } from "./fixtures/gameProfile";
import "../i18n";

vi.mock("@/lib/api", () => ({
  scanGames: vi.fn(() => Promise.resolve([testGame])),
  setBackendLanguage: vi.fn(() => Promise.resolve()),
  getGpuInfo: vi.fn(() => Promise.resolve(null)),
  isGameRunning: vi.fn(() => Promise.resolve(false)),
  launchGame: vi.fn(),
  closeGame: vi.fn(),
  openConfigFolder: vi.fn(),
  setAppBackgroundMode: vi.fn(() => Promise.resolve()),
  isTauriRuntime: () => false,
}));

function LocationProbe({ onPath }: { onPath: (path: string) => void }) {
  const { pathname } = useLocation();
  useEffect(() => {
    onPath(pathname);
  }, [pathname, onPath]);
  return null;
}

function Harness({ onPath }: { onPath: (path: string) => void }) {
  const navigate = useNavigate();
  const location = useLocation();
  const selectedGame = location.pathname.startsWith("/game/") ? testGame : null;

  return (
    <AppShell selectedGame={selectedGame}>
      <LocationProbe onPath={onPath} />
      <button type="button" onClick={() => navigate(gameTabPath(testGame.id, "advanced"))}>
        Open game
      </button>
      <Routes>
        <Route
          path="/library"
          element={
            <GameLibrary
              selectedGame={null}
              onSelectGame={(game) => navigate(gameTabPath(game.id, "advanced"))}
            />
          }
        />
        <Route path="/game/:gameId/advanced" element={<div>Editor view</div>} />
      </Routes>
    </AppShell>
  );
}

describe("library navigation from editor", () => {
  it("returns to library on first click after opening a game", async () => {
    const user = userEvent.setup();
    const paths: string[] = [];
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, staleTime: Infinity } },
    });

    render(
      <QueryClientProvider client={queryClient}>
        <AppWindowFocusProvider>
          <MemoryRouter initialEntries={[libraryPath()]}>
            <Harness onPath={(p) => paths.push(p)} />
          </MemoryRouter>
        </AppWindowFocusProvider>
      </QueryClientProvider>,
    );

    await user.click(screen.getByRole("button", { name: /open game/i }));
    await waitFor(() => expect(paths).toContain(gameTabPath(testGame.id, "advanced")));

    await user.click(screen.getByRole("link", { name: /library|библиотека/i }));

    await waitFor(() => expect(paths[paths.length - 1]).toBe(libraryPath()));
    expect(await screen.findByRole("heading", { name: /game library|библиотека/i })).toBeInTheDocument();
  });

  it("returns to library when legacy hash is on the game route", async () => {
    const user = userEvent.setup();
    const paths: string[] = [];
    const gamePath = gameTabPath(testGame.id, "advanced");
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, staleTime: Infinity } },
    });

    render(
      <QueryClientProvider client={queryClient}>
        <AppWindowFocusProvider>
          <MemoryRouter initialEntries={[`${gamePath}#basic`]}>
            <Harness onPath={(p) => paths.push(p)} />
          </MemoryRouter>
        </AppWindowFocusProvider>
      </QueryClientProvider>,
    );

    await waitFor(() => expect(paths).toContain(gamePath));

    await user.click(screen.getByRole("link", { name: /library|библиотека/i }));

    await waitFor(() => expect(paths[paths.length - 1]).toBe(libraryPath()));
  });
});
