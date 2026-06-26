import { useCallback, useEffect, useState } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import {
  type EditorFilterMode,
  type EditorPanel,
  defaultFilterMode,
  panelFromHash,
  readStoredFilterMode,
  readStoredPanel,
  writeStoredFilterMode,
  writeStoredPanel,
} from "@/lib/routing";

export function useEditorPanelState(gameId: string | undefined, defaultPanel: EditorPanel) {
  const navigate = useNavigate();
  const location = useLocation();
  const [panel, setPanelState] = useState<EditorPanel>("basic");
  const [filterMode, setFilterModeState] = useState<EditorFilterMode>("recommended");

  useEffect(() => {
    if (!gameId) return;
    const fromHash = panelFromHash(location.hash);
    const stored = readStoredPanel(gameId);
    const nextPanel = fromHash ?? stored ?? defaultPanel;
    setPanelState(nextPanel);
    if (fromHash) writeStoredPanel(gameId, nextPanel);
    const storedFilter = readStoredFilterMode(gameId, nextPanel);
    setFilterModeState(storedFilter ?? defaultFilterMode(nextPanel));

    if (location.hash) {
      navigate(
        { pathname: location.pathname, search: location.search, hash: "" },
        { replace: true },
      );
    }
    // Panel state lives in sessionStorage — do not mirror it to URL hash (breaks library nav).
    // eslint-disable-next-line react-hooks/exhaustive-deps -- init once per game open
  }, [gameId, defaultPanel]);

  const setPanel = useCallback(
    (next: EditorPanel) => {
      setPanelState(next);
      if (gameId) {
        writeStoredPanel(gameId, next);
        const storedFilter = readStoredFilterMode(gameId, next);
        setFilterModeState(storedFilter ?? defaultFilterMode(next));
      } else {
        setFilterModeState(defaultFilterMode(next));
      }
    },
    [gameId],
  );

  const setFilterMode = useCallback(
    (mode: EditorFilterMode) => {
      setFilterModeState(mode);
      if (gameId) writeStoredFilterMode(gameId, panel, mode);
    },
    [gameId, panel],
  );

  return { panel, setPanel, filterMode, setFilterMode };
}

export type EditorPanelState = ReturnType<typeof useEditorPanelState>;
