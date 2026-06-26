"""Single source of truth for supported UE catalog versions."""

from __future__ import annotations

import json
from pathlib import Path

_VERSIONS_FILE = Path(__file__).resolve().parent / "ue_versions.json"


def load_ue_versions() -> list[str]:
    data = json.loads(_VERSIONS_FILE.read_text(encoding="utf-8"))
    versions = data.get("versions")
    if not isinstance(versions, list) or not versions:
        raise ValueError(f"Invalid versions in {_VERSIONS_FILE}")
    return [str(v) for v in versions]
