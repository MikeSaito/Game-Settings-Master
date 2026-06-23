import { useRef } from "react";

/** Tracks the latest game id for async mutation callbacks. */
export function useActiveGameIdRef(gameId: string | undefined) {
  const ref = useRef(gameId);
  ref.current = gameId;
  return ref;
}
