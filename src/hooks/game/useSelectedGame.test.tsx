import { renderHook } from "@testing-library/react";
import type { ReactNode } from "react";
import { describe, expect, it } from "vitest";
import { MemoryRouter } from "react-router-dom";
import { useSelectedGame } from "@/hooks/game/useSelectedGame";
import { testGame } from "@/test/fixtures/gameProfile";

function wrapper(path: string) {
  return ({ children }: { children: ReactNode }) => (
    <MemoryRouter initialEntries={[path]}>{children}</MemoryRouter>
  );
}

describe("useSelectedGame", () => {
  it("returns game from list when on advanced route", () => {
    const { result } = renderHook(() => useSelectedGame([testGame]), {
      wrapper: wrapper(`/game/${testGame.id}/advanced`),
    });
    expect(result.current.selectedGame?.id).toBe(testGame.id);
    expect(result.current.gameRoute?.tab).toBe("advanced");
  });

  it("keeps last known game while list is empty", () => {
    const path = `/game/${testGame.id}/advanced`;
    const { result, rerender } = renderHook(
      ({ games }) => useSelectedGame(games),
      {
        wrapper: wrapper(path),
        initialProps: { games: [testGame] },
      },
    );
    expect(result.current.selectedGame?.id).toBe(testGame.id);

    rerender({ games: [] });
    expect(result.current.selectedGame?.id).toBe(testGame.id);
  });

  it("returns null on library route", () => {
    const { result } = renderHook(() => useSelectedGame([testGame]), {
      wrapper: wrapper("/library"),
    });
    expect(result.current.selectedGame).toBeNull();
    expect(result.current.gameRoute).toBeNull();
  });
});
