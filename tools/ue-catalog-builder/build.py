#!/usr/bin/env python3
"""Parse UE BaseEngine.ini / BaseScalability.ini snapshots → ue_reference_index.json."""

from __future__ import annotations

import argparse
import json
import re
import sys
from collections import defaultdict
from dataclasses import dataclass, field, asdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
REF_ROOT = ROOT / "tools" / "ue-reference"
FIXTURE_ROOT = REF_ROOT / "fixtures"
CATALOG_DIR = ROOT / "src-tauri" / "catalog"
GENERATED_DIR = CATALOG_DIR / "generated"
BUILDER_GENERATED_DIR = Path(__file__).resolve().parent / "generated"
OUTPUT_INDEX = CATALOG_DIR / "ue_reference_index.json"
OUTPUT_STATS = GENERATED_DIR / "merge_stats.json"

BUILDER_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(BUILDER_DIR))
from ue_versions import load_ue_versions

VERSIONS = load_ue_versions()
SCHEMA_VERSION = 2
CURATED_FILES = [
    "engine.json",
    "scalability.json",
    "ue4.json",
    "ue_extended.json",
    "display.json",
]

INTERESTING_ENGINE_SECTIONS = {
    "systemsettings",
    "consolevariables",
    "startup",
    "devoptions",
    "/script/engine.renderersettings",
    "/script/engine.engine",
}

SCALABILITY_GROUP_SECTION = "scalabilitygroups"
TIER_SECTION_RE = re.compile(r"^(.+?)@(\d+)$", re.IGNORECASE)
CVAR_KEY_RE = re.compile(r"^(r\.|sg\.|fx\.|t\.|p\.|a\.|s\.|g\.|d\.|m\.|b\.|c\.|f\.|n\.|i\.|u\.|v\.|w\.)", re.IGNORECASE)
BOOL_RE = re.compile(r"^(true|false)$", re.IGNORECASE)
INT_RE = re.compile(r"^-?\d+$")
FLOAT_RE = re.compile(r"^-?\d+(\.\d+)?$")

STUB_DESC_PATTERNS = [
    re.compile(r"^Стандартный UE CVar", re.IGNORECASE),
    re.compile(r"^Standard UE CVar", re.IGNORECASE),
    re.compile(r"^UE CVar \(", re.IGNORECASE),
    re.compile(r"Часто встречается в Engine\.ini", re.IGNORECASE),
    re.compile(r"Common in Engine\.ini", re.IGNORECASE),
    re.compile(r"see Unreal documentation", re.IGNORECASE),
]

HUMANIZE_ACRONYMS = frozenset(
    {
        "ao",
        "ar",
        "ai",
        "aa",
        "csm",
        "cpu",
        "dlss",
        "dof",
        "fps",
        "fsr",
        "fxaa",
        "gi",
        "gpu",
        "gtao",
        "hdr",
        "hmd",
        "lod",
        "nanite",
        "lumen",
        "rhi",
        "ssao",
        "ssr",
        "taa",
        "tsr",
        "ue",
        "ui",
        "vr",
        "vsm",
        "vsr",
        "vsync",
        "vram",
    }
)

HUMANIZE_CANONICAL_TOKENS = {
    "nanite": "Nanite",
    "lumen": "Lumen",
    "metahuman": "MetaHuman",
    "pathtracing": "Path Tracing",
    "raytracing": "Ray Tracing",
    "raytraced": "Ray Traced",
}

DESCRIPTION_OVERLAY_FIELDS = frozenset(
    {"description", "description_en", "impact", "impact_en"}
)

TEXT_SANITIZE_FIELDS = frozenset(
    {
        "title",
        "title_en",
        "description",
        "description_en",
        "impact",
        "impact_en",
        "value_hint",
        "value_hint_en",
        "ini_comment",
    }
)


def normalize_display_text(text: str) -> str:
    """Replace typography that may render as '?' in WebView2 on some Windows fonts."""
    if not text:
        return text
    return (
        text.replace("\u00ab", '"')
        .replace("\u00bb", '"')
        .replace("\u201c", '"')
        .replace("\u201d", '"')
        .replace("\u2014", " - ")
        .replace("\u2013", " - ")
        .replace("\u2192", " -> ")
    )


def sanitize_text_fields(entry: dict[str, Any]) -> None:
    for field in TEXT_SANITIZE_FIELDS:
        val = entry.get(field)
        if isinstance(val, str):
            entry[field] = normalize_display_text(val)


@dataclass
class ParsedEntry:
    key: str
    default_value: str
    section: str
    file: str
    source: str
    quality_index: int | None = None
    group_name: str | None = None


@dataclass
class MergedEntry:
    key: str
    file: str
    section: str
    value_type: str
    defaults_by_version: dict[str, str] = field(default_factory=dict)
    versions_present: list[str] = field(default_factory=list)
    introduced_in: str | None = None
    removed_in: str | None = None
    ue4: bool = False
    ue5: bool = False
    category_guess: str = "Other"
    editable: bool = True
    source: str = "BaseEngine.ini"
    quality_index: int | None = None
    group_name: str | None = None
    min: str | None = None
    max: str | None = None
    options: list[str] | None = None


@dataclass
class ScalabilityTier:
    group: str
    index: int
    section: str
    cvars: dict[str, str]
    ue_version: str


def version_folder(version: str) -> str:
    return f"UE_{version}"


def version_dir(version: str) -> Path:
    live = REF_ROOT / version_folder(version)
    if live.is_dir() and (live / "BaseEngine.ini").exists():
        return live
    return FIXTURE_ROOT / version_folder(version)


def parse_ini(path: Path) -> dict[str, dict[str, str]]:
    """Minimal UE ini parser: [Section] key=value."""
    sections: dict[str, dict[str, str]] = defaultdict(dict)
    current = ""
    if not path.exists():
        return sections
    for raw in path.read_text(encoding="utf-8-sig", errors="replace").splitlines():
        line = raw.strip()
        if not line or line.startswith(";") or line.startswith("#"):
            continue
        if line.startswith("[") and line.endswith("]"):
            current = line[1:-1].strip()
            sections[current]  # ensure key exists
            continue
        if not current:
            continue
        if "=" not in line:
            continue
        key, _, value = line.partition("=")
        key = key.strip()
        value = value.strip()
        if not key:
            continue
        sections[current][key] = value
    return dict(sections)


def is_cvar_key(key: str) -> bool:
    if key.startswith("sg."):
        return True
    return bool(CVAR_KEY_RE.match(key))


def section_interesting(section: str, file_kind: str) -> bool:
    norm = section.strip("[]").lower()
    if file_kind == "engine":
        if norm in INTERESTING_ENGINE_SECTIONS:
            return True
        if norm.startswith("/script/"):
            return True
        return norm in {"systemsettings", "consolevariables"}
    if file_kind == "scalability":
        if norm == SCALABILITY_GROUP_SECTION:
            return True
        return bool(TIER_SECTION_RE.match(norm))
    return False


def map_scalability_file_key(key: str) -> tuple[str, str]:
    """BaseScalability sg.* / tier cvars → runtime ini file."""
    if key.startswith("sg."):
        return "GameUserSettings.ini", "ScalabilityGroups"
    return "Scalability.ini", "SystemSettings"


def parse_engine_ini(version: str, path: Path) -> list[ParsedEntry]:
    out: list[ParsedEntry] = []
    for section, pairs in parse_ini(path).items():
        if not section_interesting(section, "engine"):
            continue
        for key, value in pairs.items():
            if not is_cvar_key(key):
                continue
            out.append(
                ParsedEntry(
                    key=key,
                    default_value=value,
                    section=section.strip("[]"),
                    file="Engine.ini",
                    source="BaseEngine.ini",
                )
            )
    return out


def parse_scalability_ini(version: str, path: Path) -> tuple[list[ParsedEntry], list[ScalabilityTier]]:
    entries: list[ParsedEntry] = []
    tiers: list[ScalabilityTier] = []
    for section, pairs in parse_ini(path).items():
        norm = section.strip("[]")
        lower = norm.lower()
        if lower == SCALABILITY_GROUP_SECTION:
            for key, value in pairs.items():
                if not key.startswith("sg."):
                    continue
                entries.append(
                    ParsedEntry(
                        key=key,
                        default_value=value,
                        section="ScalabilityGroups",
                        file="GameUserSettings.ini",
                        source="BaseScalability.ini",
                    )
                )
            continue
        m = TIER_SECTION_RE.match(norm)
        if not m:
            continue
        group, idx_s = m.group(1), m.group(2)
        idx = int(idx_s)
        cvars = {k: v for k, v in pairs.items() if is_cvar_key(k)}
        if not cvars:
            continue
        tiers.append(
            ScalabilityTier(
                group=group,
                index=idx,
                section=f"[{norm}]",
                cvars=cvars,
                ue_version=version,
            )
        )
        for key, value in cvars.items():
            file_name, section_name = map_scalability_file_key(key)
            entries.append(
                ParsedEntry(
                    key=key,
                    default_value=value,
                    section=section_name,
                    file=file_name,
                    source="BaseScalability.ini",
                    quality_index=idx,
                    group_name=group,
                )
            )
    return entries, tiers


def guess_value_type(value: str) -> str:
    if BOOL_RE.match(value):
        return "bool"
    if INT_RE.match(value):
        return "int"
    if FLOAT_RE.match(value):
        return "float"
    if value.lower() in {"on", "off"}:
        return "enum"
    return "string"


def guess_category(key: str, section: str) -> str:
    k = key.lower()
    s = section.lower()
    if k.startswith("sg."):
        return "Scalability"
    if k.startswith("fx."):
        return "Effects"
    if k.startswith("t."):
        return "Textures"
    if "shadow" in k:
        return "Shadows"
    if any(x in k for x in ("post", "bloom", "motion", "tonemapper", "ssr", "dof", "ambient")):
        return "PostProcess"
    if any(x in k for x in ("stream", "anisotropy", "texture", "mip")):
        return "Textures"
    if k.startswith("r."):
        return "Rendering"
    if "audio" in k or "audio" in s:
        return "Audio"
    if s.startswith("/script/") and "gameusersettings" not in s:
        return "System"
    return "Other"


def split_identifier_part(part: str) -> list[str]:
    normalized = part.replace("_", " ").replace("-", " ").strip()
    if not normalized:
        return []
    pieces: list[str] = []
    for raw in re.split(r"[.\s]+", normalized):
        if not raw:
            continue
        pieces.extend(
            p
            for p in re.findall(
                r"[A-Z]+(?=[A-Z][a-z]|\d|$)|[A-Z]?[a-z]+|\d+",
                raw,
            )
            if p
        )
    return pieces


def humanize_token(token: str) -> str:
    if not token:
        return token
    lower = token.lower()
    if lower in HUMANIZE_CANONICAL_TOKENS:
        return HUMANIZE_CANONICAL_TOKENS[lower]
    if lower in HUMANIZE_ACRONYMS:
        return lower.upper()
    if token.isupper() and len(token) > 1:
        return token
    return token[:1].upper() + token[1:]


def humanize_key(key: str) -> str:
    stripped = key
    for prefix in ("r.", "sg.", "fx.", "t.", "p."):
        if stripped.lower().startswith(prefix):
            stripped = stripped[len(prefix) :]
            break
    parts = [
        token
        for part in stripped.split(".")
        for token in split_identifier_part(part)
    ]
    return " · ".join(humanize_token(p) for p in parts if p)


def is_stub_description(text: str | None) -> bool:
    if not text or not str(text).strip():
        return True
    normalized = str(text).strip()
    return any(pattern.search(normalized) for pattern in STUB_DESC_PATTERNS)


def parse_ini_line_comments(path: Path) -> dict[str, str]:
    """Map cvar keys to ; comment text from BaseEngine.ini (line-above or inline)."""
    comments: dict[str, str] = {}
    if not path.exists():
        return comments
    pending: list[str] = []
    for raw in path.read_text(encoding="utf-8-sig", errors="replace").splitlines():
        line = raw.strip()
        if not line:
            pending = []
            continue
        if line.startswith(";"):
            text = line[1:].strip()
            if text:
                pending.append(text)
            continue
        if line.startswith("#"):
            pending = []
            continue
        if line.startswith("[") and line.endswith("]"):
            pending = []
            continue
        if "=" not in line:
            pending = []
            continue
        key, _, value = line.partition("=")
        key = key.strip()
        inline_comment = ""
        if ";" in value:
            value_part, inline_comment = value.split(";", 1)
            inline_comment = inline_comment.strip()
            value = value_part.strip()
        if not key or not is_cvar_key(key):
            pending = []
            continue
        comment_parts = list(pending)
        if inline_comment:
            comment_parts.append(inline_comment)
        if comment_parts:
            comments[key.lower()] = " ".join(comment_parts)
        pending = []
    return comments


def load_ini_comments_all_versions() -> dict[str, str]:
    merged: dict[str, str] = {}
    for version in VERSIONS:
        path = version_dir(version) / "BaseEngine.ini"
        for key, comment in parse_ini_line_comments(path).items():
            existing = merged.get(key)
            if existing is None or len(comment) > len(existing):
                merged[key] = comment
    return merged


CATEGORY_EFFECT_RU: dict[str, str] = {
    "Scalability": "меняет пресеты качества и FPS",
    "Rendering": "влияет на картинку и производительность",
    "Shadows": "влияет на тени и нагрузку на GPU",
    "Textures": "влияет на текстуры и VRAM",
    "PostProcess": "влияет на постобработку и FPS",
    "Display": "настройки экрана и окна",
    "Effects": "эффекты и дополнительная нагрузка",
    "Audio": "аудио, на FPS почти не влияет",
    "Other": "влияет на поведение движка",
}

CATEGORY_EFFECT_EN: dict[str, str] = {
    "Scalability": "changes quality presets and FPS",
    "Rendering": "affects image quality and performance",
    "Shadows": "affects shadows and GPU load",
    "Textures": "affects textures and VRAM",
    "PostProcess": "affects post-processing and FPS",
    "Display": "screen and window settings",
    "Effects": "visual effects and extra GPU load",
    "Audio": "audio settings, minimal FPS impact",
    "Other": "affects engine behavior",
}


def pick_nearest_default(m: MergedEntry, version: str = "5.4") -> str:
    if version in m.defaults_by_version:
        return m.defaults_by_version[version]
    if m.defaults_by_version:
        return next(iter(m.defaults_by_version.values()))
    return "—"


def tier_c_auto_description(
    m: MergedEntry,
    version: str = "5.4",
    ini_comment: str | None = None,
) -> tuple[str, str, str, str]:
    """Returns title_en, description_ru, description_en, value_hint."""
    cat = m.category_guess
    default = pick_nearest_default(m, version)
    file_label = m.file
    effect_ru = CATEGORY_EFFECT_RU.get(cat, CATEGORY_EFFECT_RU["Other"])
    effect_en = CATEGORY_EFFECT_EN.get(cat, CATEGORY_EFFECT_EN["Other"])
    range_ru = ""
    range_en = ""
    if m.min is not None and m.max is not None:
        range_ru = f" Допустимый диапазон: {m.min}–{m.max}."
        range_en = f" Allowed range: {m.min}–{m.max}."
    ini_ru = ""
    ini_en = ""
    if ini_comment:
        clean = ini_comment.replace("«", '"').replace("»", '"')
        ini_ru = f" Комментарий из Engine.ini: {clean}."
        ini_en = f" Engine.ini note: {clean}."
    desc_ru = (
        f'CVar "{m.key}" ({cat}, {file_label}). '
        f"Значение по умолчанию для UE {version}: {default}.{range_ru}{ini_ru} "
        f"Типичный эффект: {effect_ru}."
    )
    desc_en = (
        f'CVar "{m.key}" ({cat}, {file_label}). '
        f"Default for UE {version}: {default}.{range_en}{ini_en} "
        f"Typical effect: {effect_en}."
    )
    hint = f"{m.min}–{m.max}" if m.min and m.max else default
    return humanize_key(m.key), desc_ru, desc_en, hint


def semi_tier_b_entry(m: MergedEntry, ini_comment: str | None = None) -> dict[str, Any]:
    _, desc_ru, desc_en, hint = tier_c_auto_description(m, ini_comment=ini_comment)
    cat = m.category_guess
    return {
        "title": humanize_key(m.key),
        "title_en": humanize_key(m.key),
        "description": desc_ru,
        "description_en": desc_en,
        "impact": CATEGORY_EFFECT_RU.get(cat, CATEGORY_EFFECT_RU["Other"]),
        "impact_en": CATEGORY_EFFECT_EN.get(cat, CATEGORY_EFFECT_EN["Other"]),
        "value_hint": hint,
        "value_hint_en": hint,
    }


def expand_tier_b_by_frequency(
    merged_map: dict[str, MergedEntry],
    tier_a: dict[str, dict[str, Any]],
    tier_b_static: dict[str, dict[str, Any]],
    curated_keys: set[str],
    ini_comments: dict[str, str],
    top_n: int = 400,
) -> dict[str, dict[str, Any]]:
    """Merge static tier_b with top-N keys by version coverage (semi descriptions)."""
    merged = dict(tier_b_static)
    added = 0
    ranked = sorted(
        merged_map.values(),
        key=lambda m: (-len(m.versions_present), m.key.lower()),
    )
    for m in ranked:
        kl = m.key.lower()
        if kl in merged or kl in tier_a or kl in curated_keys:
            continue
        merged[kl] = semi_tier_b_entry(m, ini_comments.get(kl))
        added += 1
        if added >= top_n:
            break
    return merged


def resolve_description_quality(
    key: str,
    description: str,
    curated_keys: set[str],
    tier_a: dict[str, dict[str, Any]],
    tier_b: dict[str, dict[str, Any]],
    had_tier_a: bool,
    had_tier_b: bool,
) -> str:
    kl = key.lower()
    if kl in curated_keys or had_tier_a:
        return "human"
    if had_tier_b or kl in tier_b:
        if is_stub_description(description):
            return "auto"
        return "semi"
    if "see Unreal documentation" in description:
        return "auto"
    return "auto"


def count_applicable_for_version(entries: list[dict[str, Any]], version: str) -> int:
    major, minor = version_tuple(version)
    target = (major, minor)
    count = 0
    for row in entries:
        if row.get("removed_in"):
            rem = version_tuple(str(row["removed_in"]))
            if target >= rem:
                continue
        intro = row.get("introduced_in")
        if intro:
            intro_t = version_tuple(str(intro))
            if target < intro_t:
                continue
        elif row.get("versions_present"):
            if version not in row["versions_present"]:
                continue
        if row.get("ue4") and major >= 5:
            if not row.get("ue5"):
                continue
        if row.get("ue5") and major < 5:
            if not row.get("ue4"):
                continue
        count += 1
    return count


def version_tuple(version: str) -> tuple[int, int]:
    major, minor = version.split(".", 1)
    return int(major), int(minor)


def ordered_versions_present(all_versions: list[str], present: set[str]) -> list[str]:
    return [v for v in all_versions if v in present]


def compute_version_bounds(
    versions_present: list[str],
    all_versions: list[str],
    scanned_versions: set[str],
) -> tuple[str | None, str | None]:
    if not versions_present:
        return None, None
    ordered = ordered_versions_present(all_versions, set(versions_present))
    introduced = ordered[0]
    removed: str | None = None
    last = ordered[-1]
    last_idx = all_versions.index(last)
    for next_v in all_versions[last_idx + 1 :]:
        if next_v not in scanned_versions:
            break
        if next_v not in versions_present:
            removed = next_v
            break
    return introduced, removed


def infer_limits(key: str, values: list[str], value_type: str) -> tuple[str | None, str | None, list[str] | None]:
    cleaned = [v.strip() for v in values if v.strip()]
    if not cleaned:
        return None, None, None
    if value_type == "bool" or all(BOOL_RE.match(v) for v in cleaned):
        return "0", "1", ["False", "True"]
    if value_type == "enum" or all(v.lower() in {"on", "off"} for v in cleaned):
        return None, None, sorted(set(cleaned), key=str.lower)
    nums: list[float] = []
    for v in cleaned:
        try:
            nums.append(float(v))
        except ValueError:
            continue
    if not nums:
        return None, None, None
    lo, hi = min(nums), max(nums)
    if key == "sg.ResolutionQuality":
        return "25", "200", None
    if key.startswith("sg.") and key.endswith("Quality"):
        return "0", str(int(max(hi, 4))), None
    if value_type == "int":
        return str(int(lo)), str(int(hi)), None
    return f"{lo:g}", f"{hi:g}", None


def is_ue4_version(version: str) -> bool:
    return version.startswith("4.")


def merge_entries(
    by_version: dict[str, list[ParsedEntry]], all_versions: list[str]
) -> list[MergedEntry]:
    merged: dict[str, MergedEntry] = {}
    version_keys: dict[str, set[str]] = defaultdict(set)
    scanned = set(by_version.keys())

    for version, entries in by_version.items():
        ue4 = is_ue4_version(version)
        for e in entries:
            id_key = e.key.lower()
            version_keys[id_key].add(version)
            if id_key not in merged:
                merged[id_key] = MergedEntry(
                    key=e.key,
                    file=e.file,
                    section=e.section,
                    value_type=guess_value_type(e.default_value),
                    defaults_by_version={version: e.default_value},
                    versions_present=[version],
                    ue4=ue4,
                    ue5=not ue4,
                    category_guess=guess_category(e.key, e.section),
                    source=e.source,
                    quality_index=e.quality_index,
                    group_name=e.group_name,
                )
            else:
                m = merged[id_key]
                m.defaults_by_version[version] = e.default_value
                if version not in m.versions_present:
                    m.versions_present.append(version)
                if ue4:
                    m.ue4 = True
                else:
                    m.ue5 = True
                if m.section != e.section and e.section.lower() == "systemsettings":
                    m.section = e.section

    for id_key, m in merged.items():
        present = ordered_versions_present(all_versions, version_keys[id_key])
        m.versions_present = present
        m.introduced_in, m.removed_in = compute_version_bounds(present, all_versions, scanned)
        values = list(m.defaults_by_version.values())
        lo, hi, opts = infer_limits(m.key, values, m.value_type)
        m.min, m.max, m.options = lo, hi, opts

    return list(merged.values())


def load_curated_keys() -> set[str]:
    keys: set[str] = set()
    for name in CURATED_FILES:
        path = CATALOG_DIR / name
        if not path.exists():
            continue
        data = json.loads(path.read_text(encoding="utf-8"))
        for item in data:
            k = item.get("key")
            if k:
                keys.add(k.lower())
    return keys


def load_supplemental() -> list[dict[str, Any]]:
    path = Path(__file__).parent / "data" / "supplemental_cvars.json"
    if not path.exists():
        return []
    return json.loads(path.read_text(encoding="utf-8"))


def load_registry(name: str) -> dict[str, Any]:
    path = BUILDER_GENERATED_DIR / name
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def load_tier_b() -> dict[str, dict[str, Any]]:
    path = Path(__file__).parent / "data" / "tier_b_descriptions.json"
    if not path.exists():
        return {}
    data = json.loads(path.read_text(encoding="utf-8"))
    filtered: dict[str, dict[str, Any]] = {}
    for key, overlay in data.items():
        kl = key.lower()
        cleaned = dict(overlay)
        for field in ("description", "description_en", "impact", "impact_en"):
            if is_stub_description(cleaned.get(field)):
                cleaned.pop(field, None)
        if cleaned:
            filtered[kl] = cleaned
    return filtered


def load_display_overrides() -> dict[str, dict[str, Any]]:
    path = Path(__file__).parent / "data" / "display_overrides.json"
    if not path.exists():
        return {}
    data = json.loads(path.read_text(encoding="utf-8"))
    out: dict[str, dict[str, Any]] = {}
    for key, overlay in data.items():
        cleaned: dict[str, Any] = {}
        for field, val in overlay.items():
            cleaned[field] = normalize_display_text(val) if isinstance(val, str) else val
        if cleaned:
            out[key.lower()] = cleaned
    return out


def load_tier_a() -> dict[str, dict[str, Any]]:
    base = Path(__file__).parent / "data"
    tier_files = (
        "tier_a_descriptions.json",
        "tier_a_expansion_v1.json",
        "tier_a_expansion_v2.json",
        "tier_a_expansion_v3.json",
        "tier_a_expansion_v4.json",
        "tier_a_expansion_v5.json",
        "tier_a_expansion_v6.json",
    )
    out: dict[str, dict[str, Any]] = {}
    for name in tier_files:
        path = base / name
        if not path.exists():
            continue
        data = json.loads(path.read_text(encoding="utf-8"))
        for key, overlay in data.items():
            cleaned: dict[str, Any] = {}
            for field, val in overlay.items():
                cleaned[field] = normalize_display_text(val) if isinstance(val, str) else val
            out[key.lower()] = cleaned
    return out


def is_catalog_recommended(
    key: str,
    curated_keys: set[str],
    tier_a: dict[str, dict[str, Any]],
    tier_b: dict[str, dict[str, Any]],
) -> bool:
    kl = key.lower()
    if kl in curated_keys or kl in tier_a or kl in tier_b:
        return True
    if key.startswith("sg."):
        return True
    return False


def apply_tier_overlay(
    entry: dict[str, Any],
    tier: dict[str, dict[str, Any]],
) -> None:
    overlay = tier.get(entry["key"].lower())
    if not overlay:
        return
    for field in (
        "title",
        "title_en",
        "description",
        "description_en",
        "impact",
        "impact_en",
        "value_hint",
        "value_hint_en",
        "min",
        "max",
        "category",
        "value_type",
    ):
        val = overlay.get(field)
        if val is not None and val != "":
            if field in DESCRIPTION_OVERLAY_FIELDS and is_stub_description(str(val)):
                continue
            if field == "category":
                entry["category_guess"] = val
            else:
                entry[field] = (
                    normalize_display_text(str(val))
                    if field in TEXT_SANITIZE_FIELDS
                    else val
                )


def apply_supplemental(merged: dict[str, MergedEntry], all_versions: list[str]) -> None:
    for item in load_supplemental():
        key = item["key"]
        id_key = key.lower()
        if id_key in merged:
            continue
        versions = item.get("versions", ["4.27", "5.4"])
        ue4 = any(v.startswith("4.") for v in versions)
        ue5 = any(v.startswith("5.") for v in versions)
        defaults = {v: item.get("default", "0") for v in versions}
        value_type = guess_value_type(str(item.get("default", "0")))
        lo, hi, opts = infer_limits(key, list(defaults.values()), value_type)
        present = ordered_versions_present(all_versions, set(versions))
        introduced, removed = compute_version_bounds(present, all_versions, set(versions))
        merged[id_key] = MergedEntry(
            key=key,
            file=item.get("file", "Engine.ini"),
            section=item.get("section", "SystemSettings"),
            value_type=value_type,
            defaults_by_version=defaults,
            versions_present=present,
            introduced_in=introduced,
            removed_in=removed,
            ue4=ue4,
            ue5=ue5,
            category_guess=item.get("category", guess_category(key, item.get("section", ""))),
            source=item.get("source", "supplemental"),
            min=lo,
            max=hi,
            options=opts,
        )


def registry_versions(item: dict[str, Any]) -> list[str]:
    versions = item.get("versions_present") or []
    if versions:
        return ordered_versions_present(VERSIONS, set(versions))
    introduced = item.get("introduced_in")
    removed = item.get("removed_in")
    if introduced in VERSIONS:
        start = VERSIONS.index(introduced)
        end = VERSIONS.index(removed) if removed in VERSIONS else len(VERSIONS)
        return VERSIONS[start:end]
    return ["4.27", "5.4"]


def apply_sg_registry(merged: dict[str, MergedEntry], all_versions: list[str]) -> int:
    data = load_registry("sg_registry_merged.json")
    count = 0
    for item in data.get("keys", []):
        key = item.get("key")
        if not key or key.endswith(".NumLevels") or key.startswith("sg.Test."):
            continue
        count += 1
        id_key = key.lower()
        if id_key in merged:
            existing = merged[id_key]
            existing.file = "GameUserSettings.ini"
            existing.section = "ScalabilityGroups"
            existing.source = existing.source or "Scalability.cpp"
            if key == "sg.ResolutionQuality":
                existing.value_type = "float"
                existing.min = "25"
                existing.max = "200"
            elif key.startswith("sg.") and key.endswith("Quality"):
                existing.value_type = "int"
                existing.min = "0"
                if item.get("max_level") is not None:
                    existing.max = str(item["max_level"])
            continue

        versions = registry_versions(item)
        default = "100" if key == "sg.ResolutionQuality" else "3"
        value_type = "float" if key == "sg.ResolutionQuality" else "int"
        defaults = {v: default for v in versions}
        present = ordered_versions_present(all_versions, set(versions))
        introduced = item.get("introduced_in")
        removed = item.get("removed_in")
        merged[id_key] = MergedEntry(
            key=key,
            file="GameUserSettings.ini",
            section="ScalabilityGroups",
            value_type=value_type,
            defaults_by_version=defaults,
            versions_present=present,
            introduced_in=introduced,
            removed_in=removed,
            ue4=any(v.startswith("4.") for v in present),
            ue5=any(v.startswith("5.") for v in present),
            category_guess="Scalability",
            source="Scalability.cpp",
            min="25" if key == "sg.ResolutionQuality" else "0",
            max="200" if key == "sg.ResolutionQuality" else str(item.get("max_level", 3)),
        )
    return count


def gus_limits(key: str, value_type: str) -> tuple[str | None, str | None, list[str] | None]:
    if value_type == "bool":
        return "0", "1", ["False", "True"]
    if key in {"FullscreenMode", "PreferredFullscreenMode", "LastConfirmedFullscreenMode"}:
        return "0", "2", ["0", "1", "2"]
    if key.endswith("ResolutionSizeX") or key.endswith("DesiredScreenWidth"):
        return "320", "7680", None
    if key.endswith("ResolutionSizeY") or key.endswith("DesiredScreenHeight"):
        return "240", "4320", None
    if "FrameRateLimit" in key:
        return "0", "360", None
    if "AudioQualityLevel" in key:
        return "0", "4", None
    if "HDR" in key and value_type in {"int", "float"}:
        return "0", "4000", None
    return None, None, None


def apply_gus_registry(merged: dict[str, MergedEntry], all_versions: list[str], curated_keys: set[str]) -> int:
    data = load_registry("gus_registry_merged.json")
    count = 0
    for item in data.get("keys", []):
        key = item.get("key")
        if not key:
            continue
        count += 1
        id_key = key.lower()
        if id_key in merged:
            existing = merged[id_key]
            existing.file = "GameUserSettings.ini"
            existing.section = "/Script/Engine.GameUserSettings"
            continue
        versions = registry_versions(item)
        defaults = item.get("defaults_by_version") or {}
        if not defaults:
            default = str(item.get("default", "False" if item.get("value_type") == "bool" else "0"))
            defaults = {v: default for v in versions}
        present = ordered_versions_present(all_versions, set(versions))
        value_type = item.get("value_type", "string")
        lo, hi, opts = gus_limits(key, value_type)
        merged[id_key] = MergedEntry(
            key=key,
            file="GameUserSettings.ini",
            section="/Script/Engine.GameUserSettings",
            value_type=value_type,
            defaults_by_version={v: str(defaults.get(v, next(iter(defaults.values()), "0"))) for v in present},
            versions_present=present,
            introduced_in=item.get("introduced_in"),
            removed_in=item.get("removed_in"),
            ue4=any(v.startswith("4.") for v in present),
            ue5=any(v.startswith("5.") for v in present),
            category_guess=item.get("category", guess_category(key, "/Script/Engine.GameUserSettings")),
            source=item.get("source", "GameUserSettings.h"),
            min=lo,
            max=hi,
            options=opts,
        )
    return count


def collect_version(version: str) -> tuple[list[ParsedEntry], list[ScalabilityTier]]:
    vdir = version_dir(version)
    engine = vdir / "BaseEngine.ini"
    scalability = vdir / "BaseScalability.ini"
    entries = parse_engine_ini(version, engine)
    scal_entries, tiers = parse_scalability_ini(version, scalability)
    entries.extend(scal_entries)
    return entries, tiers


def build_index() -> dict[str, Any]:
    by_version: dict[str, list[ParsedEntry]] = {}
    all_tiers: list[dict[str, Any]] = []
    sources: list[str] = []

    for version in VERSIONS:
        vdir = version_dir(version)
        if not (vdir / "BaseEngine.ini").exists():
            continue
        entries, tiers = collect_version(version)
        if entries:
            by_version[version] = entries
            sources.append(version_folder(version))
        for t in tiers:
            all_tiers.append(
                {
                    "group": t.group,
                    "index": t.index,
                    "section": t.section,
                    "cvars": t.cvars,
                    "ue_version": t.ue_version,
                }
            )

    if not by_version:
        print("No reference snapshots found (local UE_* or fixtures).", file=sys.stderr)
        sys.exit(1)

    tier_a = load_tier_a()
    tier_b_static = load_tier_b()
    display_overrides = load_display_overrides()
    curated_keys = load_curated_keys()
    ini_comments = load_ini_comments_all_versions()

    merged_list = merge_entries(by_version, VERSIONS)
    merged_map = {m.key.lower(): m for m in merged_list}
    sg_registry_count = apply_sg_registry(merged_map, VERSIONS)
    gus_registry_count = apply_gus_registry(merged_map, VERSIONS, curated_keys)
    apply_supplemental(merged_map, VERSIONS)
    merged_list = sorted(merged_map.values(), key=lambda m: m.key.lower())
    tier_b = expand_tier_b_by_frequency(merged_map, tier_a, tier_b_static, curated_keys, ini_comments)

    reference_only = [m for m in merged_list if m.key.lower() not in curated_keys]
    collisions = [m for m in merged_list if m.key.lower() in curated_keys]

    introduced_stats: dict[str, int] = defaultdict(int)
    deprecated = 0
    bare_stub_count = 0
    quality_counts: dict[str, int] = defaultdict(int)
    entries_out = []
    for m in merged_list:
        if m.introduced_in:
            introduced_stats[m.introduced_in] += 1
        if m.removed_in:
            deprecated += 1
        ini_comment = ini_comments.get(m.key.lower())
        _, desc_ru, desc_en, hint = tier_c_auto_description(m, ini_comment=ini_comment)
        row: dict[str, Any] = {
            "key": m.key,
            "file": m.file,
            "section": m.section,
            "value_type": m.value_type,
            "defaults_by_version": m.defaults_by_version,
            "versions_present": m.versions_present,
            "introduced_in": m.introduced_in,
            "removed_in": m.removed_in,
            "ue4": m.ue4,
            "ue5": m.ue5,
            "category_guess": m.category_guess,
            "editable": m.editable,
            "source": m.source,
            "title": humanize_key(m.key),
            "description": desc_ru,
            "description_en": desc_en,
            "value_hint": hint,
            "value_hint_en": hint,
        }
        if ini_comment:
            row["ini_comment"] = ini_comment
        if m.min is not None:
            row["min"] = m.min
        if m.max is not None:
            row["max"] = m.max
        if m.options:
            row["options"] = [{"value": v, "label": v} for v in m.options]
        if m.quality_index is not None:
            row["quality_index"] = m.quality_index
            row["group_name"] = m.group_name
        desc_before = row["description"]
        had_a = m.key.lower() in tier_a
        had_b = m.key.lower() in tier_b
        apply_tier_overlay(row, tier_a)
        apply_tier_overlay(row, tier_b)
        apply_tier_overlay(row, display_overrides)
        if is_stub_description(row.get("description")):
            row["description"] = desc_ru
        if is_stub_description(row.get("description_en")):
            row["description_en"] = desc_en
        if is_stub_description(row.get("impact")):
            row["impact"] = CATEGORY_EFFECT_RU.get(m.category_guess, CATEGORY_EFFECT_RU["Other"])
        if is_stub_description(row.get("impact_en")):
            row["impact_en"] = CATEGORY_EFFECT_EN.get(m.category_guess, CATEGORY_EFFECT_EN["Other"])
        row["catalog_recommended"] = is_catalog_recommended(
            m.key, curated_keys, tier_a, tier_b
        )
        quality = resolve_description_quality(
            m.key,
            row["description"],
            curated_keys,
            tier_a,
            tier_b,
            had_a or row["description"] != desc_before,
            had_b,
        )
        row["description_quality"] = quality
        quality_counts[quality] += 1
        if is_stub_description(row["description"]):
            bare_stub_count += 1
        sanitize_text_fields(row)
        entries_out.append(row)

    applicable_by_version = {
        v: count_applicable_for_version(entries_out, v) for v in VERSIONS
    }

    stats = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "sources": sources,
        "schema_version": SCHEMA_VERSION,
        "total_reference_entries": len(merged_list),
        "reference_only_keys": len(reference_only),
        "curated_collisions": len(collisions),
        "curated_keys_loaded": len(curated_keys),
        "sg_registry_count": sg_registry_count,
        "gus_registry_count": gus_registry_count,
        "tier_a_overlays": len(tier_a),
        "tier_b_overlays": len(tier_b_static),
        "display_overrides": len(display_overrides),
        "tier_b_effective": len(tier_b),
        "ini_comment_keys": len(ini_comments),
        "scalability_tiers": len(all_tiers),
        "deprecated_count": deprecated,
        "introduced_by_version": dict(introduced_stats),
        "by_version_counts": {v: len(e) for v, e in by_version.items()},
        "applicable_by_version": applicable_by_version,
        "description_quality_counts": dict(quality_counts),
        "bare_stub_descriptions": bare_stub_count,
    }

    return {
        "schema_version": SCHEMA_VERSION,
        "generated_at": stats["generated_at"],
        "sources": sources,
        "entries": entries_out,
        "scalability_tiers": all_tiers,
        "stats": stats,
    }


def write_outputs(index: dict[str, Any]) -> None:
    GENERATED_DIR.mkdir(parents=True, exist_ok=True)
    stats = index.pop("stats", {})
    OUTPUT_INDEX.write_text(json.dumps(index, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    OUTPUT_STATS.write_text(json.dumps(stats, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(stats, indent=2))


def main() -> None:
    parser = argparse.ArgumentParser(description="Build UE reference catalog index")
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()
    index = build_index()
    if args.dry_run:
        print(json.dumps(index.get("stats", {}), indent=2))
        return
    write_outputs(index)


if __name__ == "__main__":
    main()
