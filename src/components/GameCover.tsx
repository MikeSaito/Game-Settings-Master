import { useEffect, useState } from "react";
import type { GameProfile } from "../lib/types";
import {
  gameCoverFallbackLetter,
  resolveGameCoverCandidates,
} from "../lib/gameCover";
import { cn } from "../lib/cn";

interface Props {
  game: GameProfile;
  className?: string;
  aspect?: "header" | "square" | "sidebar";
  selected?: boolean;
}

const aspectClass = {
  header: "aspect-[460/215]",
  square: "aspect-square",
  sidebar: "aspect-[16/9]",
};

export function GameCover({ game, className, aspect = "header", selected }: Props) {
  const coverCandidates = resolveGameCoverCandidates(game);
  const [coverIndex, setCoverIndex] = useState(0);
  const src = coverCandidates[coverIndex] ?? null;
  const letter = gameCoverFallbackLetter(game.name);

  useEffect(() => {
    setCoverIndex(0);
  }, [game.id, game.custom_cover, game.cover_url]);

  const showImage = !!src && coverIndex < coverCandidates.length;

  return (
    <div
      className={cn(
        "relative overflow-hidden rounded-lg bg-[var(--color-bg-hover)] ring-1 ring-[var(--color-border)]",
        aspectClass[aspect],
        selected && "ring-[var(--color-accent)]/50",
        className,
      )}
    >
      {showImage ? (
        <img
          src={src}
          alt=""
          className="cover-crop-center-static"
          loading="lazy"
          onError={() => {
            if (coverIndex + 1 < coverCandidates.length) {
              setCoverIndex((i) => i + 1);
            } else {
              setCoverIndex(coverCandidates.length);
            }
          }}
        />
      ) : (
        <div className="flex h-full w-full flex-col items-center justify-center bg-gradient-to-br from-[var(--color-bg-hover)] to-[var(--color-bg-card)]">
          <span className="text-2xl font-bold text-[var(--color-text-secondary)]">{letter}</span>
        </div>
      )}
      <div className="pointer-events-none absolute inset-x-0 bottom-0 h-1/2 bg-gradient-to-t from-black/50 to-transparent" />
    </div>
  );
}
