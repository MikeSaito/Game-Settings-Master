interface VirtualizerOptions {
  count: number;
  estimateSize?: () => number;
  getScrollElement?: () => Element | null;
  getItemKey?: (index: number) => string | number;
  measureElement?: (el: Element) => number;
  overscan?: number;
}

/** Lightweight stub for screenshot builds — row height must match ParameterList ROW_ESTIMATE_PX (74). */
export function useVirtualizer({ count, estimateSize = () => 74 }: VirtualizerOptions) {
  const rowHeight = estimateSize();

  return {
    getTotalSize: () => count * rowHeight,
    getVirtualItems: () =>
      Array.from({ length: count }, (_, index) => ({
        index,
        start: index * rowHeight,
        size: rowHeight,
      })),
    measureElement: () => {},
    scrollToIndex: () => {},
  };
}
