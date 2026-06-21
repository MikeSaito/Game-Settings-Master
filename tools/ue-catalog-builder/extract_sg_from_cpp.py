#!/usr/bin/env python3
"""Extract official sg.* scalability keys from Unreal Engine Scalability.cpp."""

from __future__ import annotations

import argparse
import json
import re
import sys
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
REF_ROOT = ROOT / "tools" / "ue-reference"
OUT_DIR = Path(__file__).resolve().parent / "generated"
BUILDER_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(BUILDER_DIR))
from ue_versions import load_ue_versions
DEFAULT_ENGINE_ROOTS = [
    Path("D:/UnrealEngine"),
    Path("C:/UnrealEngine"),
    Path.home() / "UnrealEngine",
]
VERSIONS = load_ue_versions()

SG_TEXT_RE = re.compile(r'TEXT\("(?P<key>sg\.[A-Za-z0-9_.]+)"\)')
NUM_LEVELS_RE = re.compile(
    r'TAutoConsoleVariable<int32>\s+\w+\s*\(\s*TEXT\("(?P<key>sg\.[A-Za-z0-9_.]+\.NumLevels)"\)\s*,\s*(?P<value>-?\d+)',
    re.MULTILINE,
)


def version_folder(version: str) -> str:
    return f"UE_{version}"


def source_path_for_version(version: str, engine_root: Path | None) -> Path | None:
    local = REF_ROOT / version_folder(version) / "source" / "Scalability.cpp"
    if local.exists():
        return local
    fixture = REF_ROOT / "fixtures" / version_folder(version) / "source" / "Scalability.cpp"
    if fixture.exists():
        return fixture
    if engine_root:
        live = engine_root / "Engine" / "Source" / "Runtime" / "Engine" / "Private" / "Scalability.cpp"
        if live.exists():
            return live
    for candidate_root in DEFAULT_ENGINE_ROOTS:
        live = candidate_root / "Engine" / "Source" / "Runtime" / "Engine" / "Private" / "Scalability.cpp"
        if live.exists():
            return live
    return None


def extract_from_file(path: Path) -> dict[str, Any]:
    text = path.read_text(encoding="utf-8-sig", errors="replace")
    raw_keys = sorted({m.group("key") for m in SG_TEXT_RE.finditer(text)})
    num_levels = {m.group("key").removesuffix(".NumLevels"): int(m.group("value")) for m in NUM_LEVELS_RE.finditer(text)}
    editable_keys = [
        key
        for key in raw_keys
        if not key.endswith(".NumLevels") and not key.startswith("sg.Test.")
    ]
    metadata_keys = [key for key in raw_keys if key.endswith(".NumLevels")]
    return {
        "keys": editable_keys,
        "metadata": {
            "num_levels": num_levels,
            "readonly_keys": metadata_keys,
        },
    }


def compute_bounds(versions_present: list[str], scanned_versions: set[str]) -> tuple[str | None, str | None]:
    if not versions_present:
        return None, None
    introduced = versions_present[0]
    removed: str | None = None
    last_idx = VERSIONS.index(versions_present[-1])
    for version in VERSIONS[last_idx + 1 :]:
        if version not in scanned_versions:
            break
        if version not in versions_present:
            removed = version
            break
    return introduced, removed


def write_json(path: Path, data: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def build_registries(versions: list[str], engine_root: Path | None, write_per_version: bool) -> dict[str, Any]:
    per_version: dict[str, dict[str, Any]] = {}
    key_versions: dict[str, set[str]] = defaultdict(set)
    num_levels_by_key: dict[str, dict[str, int]] = defaultdict(dict)
    scanned_versions: set[str] = set()
    sources: dict[str, str] = {}

    for version in versions:
        source = source_path_for_version(version, engine_root)
        if source is None:
            continue
        extracted = extract_from_file(source)
        scanned_versions.add(version)
        sources[version] = str(source)
        per_version[version] = {
            "version": version,
            "keys": extracted["keys"],
            "source": "Scalability.cpp",
            "source_path": str(source),
            "metadata": extracted["metadata"],
        }
        for key in extracted["keys"]:
            key_versions[key].add(version)
        for key, num_levels in extracted["metadata"]["num_levels"].items():
            num_levels_by_key[key][version] = num_levels
        if write_per_version:
            write_json(OUT_DIR / f"sg_registry_UE_{version}.json", per_version[version])

    merged_keys = []
    for key in sorted(key_versions, key=str.lower):
        versions_present = [version for version in VERSIONS if version in key_versions[key]]
        introduced, removed = compute_bounds(versions_present, scanned_versions)
        row: dict[str, Any] = {
            "key": key,
            "introduced_in": introduced,
            "removed_in": removed,
            "versions_present": versions_present,
            "source": "Scalability.cpp",
            "editable": True,
        }
        if key in num_levels_by_key:
            row["num_levels_by_version"] = {
                version: num_levels_by_key[key][version]
                for version in VERSIONS
                if version in num_levels_by_key[key]
            }
            row["max_level"] = max(num_levels_by_key[key].values()) - 1
        merged_keys.append(row)

    merged = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "source": "Scalability.cpp",
        "versions_scanned": [version for version in VERSIONS if version in scanned_versions],
        "sources": sources,
        "keys": merged_keys,
    }
    write_json(OUT_DIR / "sg_registry_merged.json", merged)
    return merged


def main() -> None:
    parser = argparse.ArgumentParser(description="Extract sg.* keys from Unreal Scalability.cpp")
    parser.add_argument("--all-versions", action="store_true", help="scan all supported UE versions")
    parser.add_argument("--version", action="append", dest="versions", help="UE version to scan, repeatable")
    parser.add_argument("--engine-root", type=Path, default=None, help="path to a local UnrealEngine clone")
    parser.add_argument("--no-per-version", action="store_true", help="only write sg_registry_merged.json")
    args = parser.parse_args()

    versions = VERSIONS if args.all_versions or not args.versions else args.versions
    merged = build_registries(versions, args.engine_root, not args.no_per_version)
    if not merged["versions_scanned"]:
        print("No Scalability.cpp snapshots found.", file=sys.stderr)
        sys.exit(1)
    print(json.dumps({"versions_scanned": merged["versions_scanned"], "sg_registry_count": len(merged["keys"])}, indent=2))


if __name__ == "__main__":
    main()
