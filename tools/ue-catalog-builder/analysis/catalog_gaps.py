#!/usr/bin/env python3
"""Print catalog description quality gaps for roadmap tracking."""

from __future__ import annotations

import json
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
INDEX = ROOT / "src-tauri" / "catalog" / "ue_reference_index.json"
STATS = ROOT / "src-tauri" / "catalog" / "generated" / "merge_stats.json"


def main() -> None:
    stats = json.loads(STATS.read_text(encoding="utf-8"))
    data = json.loads(INDEX.read_text(encoding="utf-8"))
    entries = data["entries"]

    print("=== merge_stats ===")
    print(json.dumps(stats.get("description_quality_counts"), ensure_ascii=False, indent=2))
    print(f"tier_a_overlays: {stats.get('tier_a_overlays')}")
    print(f"tier_b_overlays: {stats.get('tier_b_overlays')}")

    by_quality: dict[str, list[str]] = {"human": [], "semi": [], "auto": []}
    for row in entries:
        q = row.get("description_quality", "auto")
        by_quality.setdefault(q, []).append(row["key"])

    print(f"\n=== auto keys ({len(by_quality['auto'])}) — top prefixes ===")
    prefixes = Counter()
    for key in by_quality["auto"]:
        head = key.split(".")[0] + "." + (key.split(".")[1] if "." in key else "")
        prefixes[head] += 1
    for prefix, count in prefixes.most_common(15):
        print(f"  {prefix}: {count}")

    print("\n=== sample auto (first 25) ===")
    for key in sorted(by_quality["auto"])[:25]:
        print(f"  {key}")


if __name__ == "__main__":
    main()
