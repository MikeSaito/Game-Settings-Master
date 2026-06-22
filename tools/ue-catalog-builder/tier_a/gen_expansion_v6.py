#!/usr/bin/env python3
"""Generate tier_a_expansion_v6.json from generic catalog keys."""

from __future__ import annotations

import json
import sys
from pathlib import Path

TIER_A_DIR = Path(__file__).resolve().parent
BUILDER_DIR = TIER_A_DIR.parent
sys.path.insert(0, str(TIER_A_DIR))

from v6_families import FAMILY, SUFFIX, build_family_entry  # noqa: E402
from v6_lumen import LUMEN  # noqa: E402
from v6_misc import build_misc  # noqa: E402

OUT = BUILDER_DIR / "data" / "tier_a_expansion_v6.json"
INDEX = BUILDER_DIR.parents[1] / "src-tauri" / "catalog" / "ue_reference_index.json"
DATA = BUILDER_DIR / "data"

# Tier B static keys upgraded to tier A (not in generic list)
EXTRA_KEYS = ("r.Render.Quality",)

TEMPLATE_SUFFIX = {
    "Bias", "Count", "Distance", "Enable", "Intensity", "Max", "Min",
    "Quality", "Resolution", "Scale", "Size", "Threshold",
}
TIER_C = "Typical effect:"


def load_existing_tier_a() -> set[str]:
    keys: set[str] = set()
    for name in (
        "tier_a_descriptions.json",
        "tier_a_expansion_v1.json",
        "tier_a_expansion_v2.json",
        "tier_a_expansion_v3.json",
        "tier_a_expansion_v4.json",
        "tier_a_expansion_v5.json",
    ):
        path = DATA / name
        if path.exists():
            keys.update(json.loads(path.read_text(encoding="utf-8")).keys())
    return {k.lower() for k in keys}


def keys_for_v6() -> list[str]:
    """Keys not yet covered by tier_a v1–v5 (stable across rebuilds)."""
    prior = load_existing_tier_a()
    entries = json.loads(INDEX.read_text(encoding="utf-8"))["entries"]
    out: list[str] = []
    seen: set[str] = set()
    for e in entries:
        k = e["key"]
        kl = k.lower()
        if kl in prior or kl in seen:
            continue
        out.append(k)
        seen.add(kl)
    for key in EXTRA_KEYS:
        if key.lower() not in prior and key.lower() not in seen:
            out.append(key)
            seen.add(key.lower())
    return sorted(out)


def try_family(key: str) -> dict | None:
    parts = key.split(".")
    if len(parts) != 3 or parts[-1] not in TEMPLATE_SUFFIX:
        return None
    family = parts[1]
    if family not in FAMILY:
        return None
    return build_family_entry(key)


def build_entry(key: str) -> dict:
    if key in LUMEN:
        return LUMEN[key]
    fam = try_family(key)
    if fam:
        return fam
    misc = build_misc(key)
    if misc:
        return misc
    raise ValueError(f"no v6 entry for {key}")


def main() -> None:
    keys = keys_for_v6()
    out: dict[str, dict] = {}
    missing: list[str] = []
    for key in keys:
        try:
            out[key] = build_entry(key)
        except ValueError:
            missing.append(key)
    if missing:
        print(f"MISSING {len(missing)}:", *missing[:20], sep="\n  ")
        raise SystemExit(1)
    OUT.write_text(json.dumps(out, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"wrote {len(out)} -> {OUT}")


if __name__ == "__main__":
    main()
