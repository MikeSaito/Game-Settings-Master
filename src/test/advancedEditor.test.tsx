import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { AppWindowFocusProvider } from "../context/AppWindowFocusProvider";
import { ParameterList } from "../components/advanced/ParameterList";
import { testParameters } from "./fixtures/testParameters";
import "../i18n";

vi.mock("@tanstack/react-virtual", () => ({
  useVirtualizer: ({ count }: { count: number }) => {
    const visibleCount = Math.min(count, 8);
    const ROW_ESTIMATE_PX = 120;
    return {
      getTotalSize: () => count * ROW_ESTIMATE_PX,
      getVirtualItems: () =>
        Array.from({ length: visibleCount }, (_, index) => ({
          index,
          start: index * ROW_ESTIMATE_PX,
        })),
      measureElement: () => {},
    };
  },
}));

function renderAdvancedEditor() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <AppWindowFocusProvider>
        <ParameterList
          filteredParams={testParameters}
          search=""
          parametersLoading={false}
          gpu={undefined}
          engineEnabled={new Set()}
          onUpdateParam={() => {}}
          onToggleEngineParam={() => {}}
        />
      </AppWindowFocusProvider>
    </QueryClientProvider>,
  );
}

describe("AdvancedEditor integration", () => {
  it("renders virtualized parameter list with full count", async () => {
    renderAdvancedEditor();
    const virtualLists = await screen.findAllByTestId("parameter-list-virtual");
    const virtualList = virtualLists[0];
    expect(virtualList).toHaveAttribute(
      "data-virtual-count",
      String(testParameters.length),
    );
  });

  it("mounts fewer DOM rows than total parameters (virtualization)", async () => {
    renderAdvancedEditor();
    const virtualLists = await screen.findAllByTestId("parameter-list-virtual");
    const virtualList = virtualLists[0];
    const renderedRows = within(virtualList).queryAllByText(/^Parameter \d+$/);
    expect(renderedRows.length).toBeGreaterThan(0);
    expect(renderedRows.length).toBeLessThan(testParameters.length);
  });

  it("renders scroll container and a parameter row", async () => {
    renderAdvancedEditor();
    const scrollEls = await screen.findAllByTestId("parameter-list-scroll");
    expect(scrollEls.length).toBeGreaterThan(0);
    const param0Els = await screen.findAllByText("Parameter 0");
    expect(param0Els.length).toBeGreaterThan(0);
  });
});
