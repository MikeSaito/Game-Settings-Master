import { useEffect, useState } from "react";
import type { GameProfile } from "@/lib/core";
import {
  gameCoverFallbackLetter,
  resolveGameCoverCandidates,
} from "@/lib/game";
import { cn } from "@/lib/core";

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
        "relative overflow-hidden rounded-[var(--radius-control)] bg-[var(--color-surface-hover)] ring-1 ring-[var(--color-border)]",
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
        <div className="flex h-full w-full flex-col items-center justify-center bg-gradient-to-br from-[var(--color-surface-hover)] to-[var(--color-surface)]">
          <span className="text-2xl font-bold text-[var(--color-text-secondary)]">{letter}</span>
        </div>
      )}
      <div className="pointer-events-none absolute inset-x-0 bottom-0 h-1/2 bg-gradient-to-t from-black/50 to-transparent" />
    </div>
  );
}
