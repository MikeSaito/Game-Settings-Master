#!/usr/bin/env python3
"""Extract UGameUserSettings config fields from Unreal Engine headers."""

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
BUILDER_DIR = Path(__file__).resolve().parents[1]
OUT_DIR = BUILDER_DIR / "generated"
sys.path.insert(0, str(BUILDER_DIR / "shared"))
from ue_versions import load_ue_versions
DEFAULT_ENGINE_ROOTS = [
    Path("D:/UnrealEngine"),
    Path("C:/UnrealEngine"),
    Path.home() / "UnrealEngine",
]
VERSIONS = load_ue_versions()
GUS_SECTION = "/Script/Engine.GameUserSettings"

PROPERTY_RE = re.compile(
    r'UPROPERTY\s*\((?P<meta>[^)]*(?:\bconfig\b|\bglobalconfig\b)[^)]*)\)\s*(?:\r?\n\s*(?:/\*\*.*?\*/\s*)?)*(?P<type>[A-Za-z_][A-Za-z0-9_:<>]*)\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*;',
    re.IGNORECASE | re.DOTALL,
)
ASSIGNMENT_RE = re.compile(r"\b(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*=\s*(?P<value>[^;\n]+);")
CHAIN_ASSIGNMENT_RE = re.compile(
    r"\b(?P<left>[A-Za-z_][A-Za-z0-9_]*)\s*=\s*(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*=\s*(?P<value>[^;\n]+);"
)
SET_DEFAULTS_RE = re.compile(
    r"void\s+UGameUserSettings::SetToDefaults\s*\([^)]*\)\s*\{(?P<body>.*?)\n\}",
    re.DOTALL,
)


def version_folder(version: str) -> str:
    return f"UE_{version}"


def source_paths_for_version(version: str, engine_root: Path | None) -> tuple[Path | None, Path | None]:
    compact_candidates = [
        REF_ROOT / version_folder(version) / "source",
        REF_ROOT / "fixtures" / version_folder(version) / "source",
    ]
    for base in compact_candidates:
        header = base / "GameUserSettings.h"
        cpp = base / "GameUserSettings.cpp"
        if header.exists():
            return header, cpp if cpp.exists() else None

    candidates: list[Path] = []
    if engine_root:
        candidates.append(engine_root / "Engine" / "Source" / "Runtime" / "Engine")
    candidates.extend(root / "Engine" / "Source" / "Runtime" / "Engine" for root in DEFAULT_ENGINE_ROOTS)

    for base in candidates:
        header = base / "Classes" / "GameFramework" / "GameUserSettings.h"
        if not header.exists():
            header = base / "Public" / "GameFramework" / "GameUserSettings.h"
        cpp = base / "Private" / "GameUserSettings.cpp"
        if header.exists():
            return header, cpp if cpp.exists() else None
    return None, None


def cpp_type_to_value_type(cpp_type: str) -> str:
    normalized = cpp_type.replace("::", "").lower()
    if normalized == "bool":
        return "bool"
    if normalized in {"float", "double"}:
        return "float"
    if normalized.startswith(("int", "uint")):
        return "int"
    if normalized in {"fstring", "fname", "ftext"}:
        return "string"
    if normalized.startswith("tarray"):
        return "array"
    return "string"


def category_for_field(name: str, cpp_type: str) -> str:
    lower = name.lower()
    if "audio" in lower:
        return "Audio"
    if "hdr" in lower:
        return "Display"
    if "fullscreen" in lower or "resolution" in lower or "screen" in lower or "display" in lower:
        return "Display"
    if "window" in lower:
        return "Window"
    if "benchmark" in lower or "framerate" in lower or "dynamic" in lower:
        return "Performance"
    if cpp_type.lower().startswith("tarray"):
        return "Performance"
    return "Display"


def clean_default(value: str) -> str:
    value = value.strip()
    value = re.sub(r"\s+", " ", value)
    if value.endswith("f"):
        value = value[:-1]
    if value in {"true", "false"}:
        return value[:1].upper() + value[1:]
    return value


def extract_defaults(cpp_path: Path | None) -> dict[str, str]:
    if cpp_path is None or not cpp_path.exists():
        return {}
    text = cpp_path.read_text(encoding="utf-8-sig", errors="replace")
    match = SET_DEFAULTS_RE.search(text)
    if not match:
        return {}
    body = match.group("body")
    defaults: dict[str, str] = {}
    for match in CHAIN_ASSIGNMENT_RE.finditer(body):
        value = clean_default(match.group("value"))
        defaults[match.group("left")] = value
        defaults[match.group("name")] = value
    for match in ASSIGNMENT_RE.finditer(body):
        name = match.group("name")
        if name not in defaults:
            defaults[name] = clean_default(match.group("value"))
    return defaults


def extract_from_files(header_path: Path, cpp_path: Path | None) -> list[dict[str, Any]]:
    text = header_path.read_text(encoding="utf-8-sig", errors="replace")
    defaults = extract_defaults(cpp_path)
    fields: list[dict[str, Any]] = []
    seen: set[str] = set()
    for match in PROPERTY_RE.finditer(text):
        meta = match.group("meta")
        cpp_type = match.group("type")
        name = match.group("name")
        if name in seen:
            continue
        seen.add(name)
        row: dict[str, Any] = {
            "key": name,
            "cpp_type": cpp_type,
            "value_type": cpp_type_to_value_type(cpp_type),
            "category": category_for_field(name, cpp_type),
            "file": "GameUserSettings.ini",
            "section": GUS_SECTION,
            "source": "GameUserSettings.h",
            "config_scope": "globalconfig" if "globalconfig" in meta.lower() else "config",
        }
        if name in defaults:
            row["default"] = defaults[name]
        fields.append(row)
    return fields


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
    per_version: dict[str, list[dict[str, Any]]] = {}
    key_versions: dict[str, set[str]] = defaultdict(set)
    key_rows: dict[str, dict[str, dict[str, Any]]] = defaultdict(dict)
    scanned_versions: set[str] = set()
    sources: dict[str, str] = {}

    for version in versions:
        header, cpp = source_paths_for_version(version, engine_root)
        if header is None:
            continue
        fields = extract_from_files(header, cpp)
        scanned_versions.add(version)
        sources[version] = str(header)
        per_version[version] = fields
        for field in fields:
            key = field["key"]
            key_versions[key].add(version)
            key_rows[key][version] = field
        if write_per_version:
            write_json(
                OUT_DIR / f"gus_registry_UE_{version}.json",
                {
                    "version": version,
                    "keys": fields,
                    "source": "GameUserSettings.h",
                    "source_path": str(header),
                    "defaults_source_path": str(cpp) if cpp else None,
                },
            )

    merged_keys: list[dict[str, Any]] = []
    for key in sorted(key_versions, key=str.lower):
        versions_present = [version for version in VERSIONS if version in key_versions[key]]
        introduced, removed = compute_bounds(versions_present, scanned_versions)
        latest_version = versions_present[-1]
        latest = dict(key_rows[key][latest_version])
        latest["introduced_in"] = introduced
        latest["removed_in"] = removed
        latest["versions_present"] = versions_present
        latest["defaults_by_version"] = {
            version: key_rows[key][version]["default"]
            for version in versions_present
            if "default" in key_rows[key][version]
        }
        merged_keys.append(latest)

    merged = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "source": "GameUserSettings.h",
        "versions_scanned": [version for version in VERSIONS if version in scanned_versions],
        "sources": sources,
        "keys": merged_keys,
    }
    write_json(OUT_DIR / "gus_registry_merged.json", merged)
    return merged


def main() -> None:
    parser = argparse.ArgumentParser(description="Extract UGameUserSettings config fields")
    parser.add_argument("--all-versions", action="store_true", help="scan all supported UE versions")
    parser.add_argument("--version", action="append", dest="versions", help="UE version to scan, repeatable")
    parser.add_argument("--engine-root", type=Path, default=None, help="path to a local UnrealEngine clone")
    parser.add_argument("--no-per-version", action="store_true", help="only write gus_registry_merged.json")
    args = parser.parse_args()

    versions = VERSIONS if args.all_versions or not args.versions else args.versions
    merged = build_registries(versions, args.engine_root, not args.no_per_version)
    if not merged["versions_scanned"]:
        print("No GameUserSettings.h snapshots found.", file=sys.stderr)
        sys.exit(1)
    print(json.dumps({"versions_scanned": merged["versions_scanned"], "gus_registry_count": len(merged["keys"])}, indent=2))


if __name__ == "__main__":
    main()
