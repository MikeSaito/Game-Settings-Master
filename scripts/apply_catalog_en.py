#!/usr/bin/env python3
"""Add title_en / description_en / impact_en / value_hint_en to catalog JSON files."""
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TRANSLATIONS_PATH = Path(__file__).resolve().parent / "catalog_en_translations.json"

CATALOG_TARGETS: list[tuple[Path, str]] = [
    (ROOT / "src-tauri" / "catalog", "bundled"),
    (ROOT / "vps" / "source" / "ue-catalog", "vps-mirror"),
]

FORZA_PATH = ROOT / "vps" / "source" / "forza-fh6" / "parameter-catalog.json"
FORZA_FILENAME = "parameter-catalog.json"

# Field order: Russian text fields, then English, then the rest.
TEXT_FIELDS = ("title", "description", "impact", "value_hint")
EN_FIELDS = ("title_en", "description_en", "impact_en", "value_hint_en")


def load_translations() -> tuple[dict, dict]:
    data = json.loads(TRANSLATIONS_PATH.read_text(encoding="utf-8"))
    return data.get("by_key", {}), data.get("file_overrides", {})


def lookup_translation(
    filename: str,
    entry: dict,
    by_key: dict,
    file_overrides: dict,
) -> dict | None:
    key = entry.get("key", "")
    override_key = f"{filename}::{key}"
    if override_key in file_overrides:
        return file_overrides[override_key]
    return by_key.get(key)


def enrich_entry(
    filename: str,
    entry: dict,
    by_key: dict,
    file_overrides: dict,
) -> dict:
    tr = lookup_translation(filename, entry, by_key, file_overrides)
    if not tr:
        missing = f"{filename}::{entry.get('key', '?')}"
        raise KeyError(f"No English translation for {missing}")

    out: dict = {}
    for field, value in entry.items():
        if field in EN_FIELDS:
            continue
        out[field] = value
        if field == "title" and "title_en" in tr:
            out["title_en"] = tr["title_en"]
        elif field == "description" and "description_en" in tr:
            out["description_en"] = tr["description_en"]
        elif field == "impact" and entry.get("impact") and "impact_en" in tr:
            out["impact_en"] = tr["impact_en"]
        elif field == "value_hint" and entry.get("value_hint") and "value_hint_en" in tr:
            out["value_hint_en"] = tr["value_hint_en"]

    # Reorder for readability: title, title_en, description, description_en, ...
    ordered: dict = {}
    for field in entry:
        if field in EN_FIELDS:
            continue
        ordered[field] = out[field]
        en_field = f"{field}_en" if field in TEXT_FIELDS else None
        if en_field and en_field in out:
            ordered[en_field] = out[en_field]

    return ordered


def process_file(
    path: Path,
    filename: str,
    by_key: dict,
    file_overrides: dict,
) -> list[dict]:
    entries = json.loads(path.read_text(encoding="utf-8"))
    return [enrich_entry(filename, e, by_key, file_overrides) for e in entries]


def write_catalog(path: Path, entries: list[dict]) -> None:
    text = json.dumps(entries, ensure_ascii=False, indent=2)
    path.write_text(text + "\n", encoding="utf-8")


def main() -> int:
    by_key, file_overrides = load_translations()
    counts: dict[str, int] = {}

    for catalog_dir, label in CATALOG_TARGETS:
        if not catalog_dir.is_dir():
            print(f"ERROR: missing {label} dir: {catalog_dir}", file=sys.stderr)
            return 1
        for json_path in sorted(catalog_dir.glob("*.json")):
            enriched = process_file(json_path, json_path.name, by_key, file_overrides)
            write_catalog(json_path, enriched)
            counts[f"{label}/{json_path.name}"] = len(enriched)
            print(f"  {label}: {json_path.name} — {len(enriched)} entries")

    if FORZA_PATH.is_file():
        enriched = process_file(FORZA_PATH, FORZA_FILENAME, by_key, file_overrides)
        write_catalog(FORZA_PATH, enriched)
        counts[f"forza/{FORZA_FILENAME}"] = len(enriched)
        print(f"  forza: {FORZA_FILENAME} — {len(enriched)} entries")
    else:
        print("  forza: parameter-catalog.json not found (skipped)")

    print("\nSummary:")
    for name, n in sorted(counts.items()):
        print(f"  {name}: {n}")

    total = sum(counts.values())
    print(f"\nTotal entries updated: {total}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
