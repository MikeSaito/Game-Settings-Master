#!/usr/bin/env python3
"""Analyze what remains in the catalog roadmap."""

from __future__ import annotations

import json
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
INDEX = ROOT / "src-tauri" / "catalog" / "ue_reference_index.json"
STATS = ROOT / "src-tauri" / "catalog" / "generated" / "merge_stats.json"
DATA = Path(__file__).resolve().parent / "data"
CATALOG = ROOT / "src-tauri" / "catalog"

TIER_C_RU = "Типичный эффект:"
TIER_C_EN = "Typical effect:"


def load_json(path: Path) -> dict | list:
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> None:
    entries = load_json(INDEX)["entries"]
    stats = load_json(STATS)

    curated_keys: set[str] = set()
    for name in (
        "engine.json",
        "scalability.json",
        "display.json",
        "ue4.json",
        "ue_extended.json",
    ):
        data = load_json(CATALOG / name)
        if isinstance(data, list):
            for item in data:
                k = item.get("key")
                if k:
                    curated_keys.add(k.lower())

    tier_a: dict = {}
    for name in (
        "tier_a_descriptions.json",
        "tier_a_expansion_v1.json",
        "tier_a_expansion_v2.json",
        "tier_a_expansion_v3.json",
        "tier_a_expansion_v4.json",
        "tier_a_expansion_v5.json",
        "tier_a_expansion_v6.json",
    ):
        data = load_json(DATA / name)
        if isinstance(data, dict):
            tier_a.update({k.lower(): v for k, v in data.items()})

    tier_b_static = load_json(DATA / "tier_b_descriptions.json")
    tier_b_keys = {k.lower() for k in tier_b_static} if isinstance(tier_b_static, dict) else set()

    by_source: Counter[str] = Counter()
    no_tier_a: list[str] = []
    generic_human: list[str] = []
    tier_b_only: list[str] = []

    for e in entries:
        k = e["key"].lower()
        desc = e.get("description", "")
        in_c = k in curated_keys
        in_a = k in tier_a
        in_b = k in tier_b_keys
        is_generic = TIER_C_EN in e.get("description_en", "") or TIER_C_RU in desc

        if in_c and in_a:
            by_source["curated + tier_a"] += 1
        elif in_c:
            by_source["curated only"] += 1
        elif in_a:
            by_source["tier_a only"] += 1
        elif in_b:
            by_source["tier_b static only"] += 1
        elif is_generic:
            by_source["tier_b freq (generic template)"] += 1
        else:
            by_source["other"] += 1

        if not in_a:
            no_tier_a.append(e["key"])
        if is_generic and not in_a and not in_c:
            generic_human.append(e["key"])
        if in_b and not in_a and not in_c:
            tier_b_only.append(e["key"])

    not_recommended = [e["key"] for e in entries if not e.get("catalog_recommended")]
    deprecated = [e["key"] for e in entries if e.get("removed_in")]

    print("=== QUALITY (badge) ===")
    print(json.dumps(stats.get("description_quality_counts"), ensure_ascii=False, indent=2))
    print(f"total entries: {len(entries)}")
    print(f"bare_stub_descriptions: {stats.get('bare_stub_descriptions')}")
    print(f"tier_a_overlays: {stats.get('tier_a_overlays')}")
    print(f"tier_b_overlays: {stats.get('tier_b_overlays')} (effective {stats.get('tier_b_effective')})")

    print("\n=== OVERLAY SOURCE ===")
    for label, count in by_source.most_common():
        print(f"  {label}: {count}")

    print(f"\n=== WITHOUT tier_a ({len(no_tier_a)}) — upgrade candidates ===")
    pfx = Counter(
        x.split(".")[0] + "." + (x.split(".")[1] if "." in x else "")
        for x in no_tier_a
    )
    for prefix, count in pfx.most_common(15):
        print(f"  {prefix}: {count}")

    print(f"\n=== Generic tier_c text, still human ({len(generic_human)}) ===")
    pfx2 = Counter(
        x.split(".")[0] + "." + (x.split(".")[1] if "." in x else "")
        for x in generic_human
    )
    for prefix, count in pfx2.most_common(15):
        print(f"  {prefix}: {count}")
    print("  sample:")
    for key in sorted(generic_human)[:20]:
        print(f"    {key}")

    print(f"\n=== tier_b static only, no tier_a ({len(tier_b_only)}) ===")
    for key in sorted(tier_b_only)[:20]:
        print(f"  {key}")

    print(f"\n=== catalog_recommended=false ({len(not_recommended)}) ===")
    if not_recommended:
        print("  (keys in index but not curated/tier_a/tier_b/sg)")
        for key in not_recommended[:15]:
            print(f"    {key}")

    print(f"\n=== deprecated removed_in ({len(deprecated)}) ===")
    for key in deprecated[:20]:
        print(f"  {key}")

    print("\n=== UE VERSION COVERAGE ===")
    print(f"  sources: {stats.get('sources')}")
    abv = stats.get("applicable_by_version", {})
    for v in stats.get("sources", []):
        print(f"  {v}: {abv.get(v, '?')} applicable keys")

    print("\n=== OUT OF SCOPE (needs Epic clone / new UE tag) ===")
    print("  fixtures ~726 keys from BaseEngine.ini + BaseScalability.ini (4.27–5.8)")
    print("  full Epic git clone can add 1000+ keys — see docs/epic-clone-setup.md")
    print("  UE 5.9+: add to tools/ue-catalog-builder/ue_versions.json + fetch")


if __name__ == "__main__":
    main()
