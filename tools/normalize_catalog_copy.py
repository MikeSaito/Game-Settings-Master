#!/usr/bin/env python3
"""Normalize catalog titles and typography in user-facing copy."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CATALOG = ROOT / "src-tauri" / "catalog"
I18N = ROOT / "src" / "i18n" / "locales"
BUILDER_DATA = ROOT / "tools" / "ue-catalog-builder" / "data"

TEXT_FIELDS = {
    "title",
    "title_en",
    "description",
    "description_en",
    "impact",
    "impact_en",
    "value_hint",
    "value_hint_en",
}

TITLE_FIXES_RU = {
    "Как далеко от вас видны тени": "Дальность теней",
    "Как свет заполняет углы и темные места": "Глобальное освещение",
    "Насколько «живыми» выглядят материалы": "Качество шейдинга",
    'Насколько "живыми" выглядят материалы': "Качество шейдинга",
    "Насколько далеко видны объекты (скрытый множитель)": "Масштаб дальности обзора",
    "Насколько четкие края теней": "Разрешение теней",
    "Насколько гладкие края объектов": "Сила сглаживания",
    "Насколько широкие тени в углах": "Контактные тени (SSAO)",
    "Насколько сильно размывается картинка в движении": "Размытие в движении",
    "DLSS – качество и FPS": "Режим DLSS",
    "TSR – upscaling без NVIDIA": "Режим TSR",
    "Четкость после увеличения картинки (DLSS/TSR)": "Резкость upscaling (DLSS/TSR)",
    "Как игра рисует меньше, но показывает четко (DLSS)": "Режим DLSS",
    "Как убираются зубцы на краях объектов": "Сглаживание",
    "Как игра увеличивает картинку для FPS": "Метод upscaling",
    "Насколько «широко» вы видите мир": "Угол обзора (FOV)",
    "Upscaling без NVIDIA (TSR) – для любой видеокарты": "Режим TSR",
    "DLSS – то же, что режим выше, но числом": "Режим DLSS (число)",
}

TITLE_FIXES_EN = {
    "How far shadows are visible": "Shadow distance",
    "How light fills corners and dark areas": "Global illumination",
    "How alive materials look": "Shading quality",
    "How the game renders less but looks sharp (DLSS)": "DLSS mode",
    "How the game upscales for FPS": "Upscaling method",
    "How jagged edges are smoothed": "Anti-aliasing type",
    "Sharpness after upscaling (DLSS/TSR)": "Upscaling sharpness (DLSS/TSR)",
    "How wide you see the world": "Field of view (FOV)",
}

DESC_REPLACEMENTS = [
    (
        "Как игра убирает «зубцы» на краях объектов, когда вы двигаетесь. ",
        "Сглаживание краев в движении. ",
    ),
    (
        "Как игра «доводит» яркий кадр до того, что вы видите на мониторе: ",
        "Тоновое отображение на экране: ",
    ),
    (
        "Как в кино: то, что далеко или очень близко к камере, слегка размывается, а фокус ",
        "Размывает то, что далеко или очень близко к камере; фокус ",
    ),
]


def fix_title(field: str, value: str) -> str:
    if field == "title":
        return TITLE_FIXES_RU.get(value, value)
    if field == "title_en":
        return TITLE_FIXES_EN.get(value, value)
    return value


def normalize_copy_style(text: str) -> str:
    return text.translate({0x2014: 0x2013, 0x0451: 0x0435, 0x0401: 0x0415})


def unsupported_characters(text: str) -> int:
    return sum(text.count(chr(code)) for code in (0x2014, 0x0451, 0x0401))


def normalize_text_field(field: str, value: str) -> str:
    if not isinstance(value, str):
        return value
    out = fix_title(field, normalize_copy_style(value))
    if field.startswith("description"):
        for old, new in DESC_REPLACEMENTS:
            out = out.replace(old, new)
    return out


def walk(obj) -> None:
    if isinstance(obj, dict):
        for key, value in obj.items():
            if key in TEXT_FIELDS and isinstance(value, str):
                obj[key] = normalize_text_field(key, value)
            else:
                walk(value)
    elif isinstance(obj, list):
        for item in obj:
            walk(item)


def process_json_file(path: Path) -> int:
    raw = path.read_text(encoding="utf-8")
    before = unsupported_characters(raw)
    data = json.loads(raw)
    walk(data)
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    after = unsupported_characters(path.read_text(encoding="utf-8"))
    return before - after


def process_i18n_file(path: Path) -> int:
    raw = path.read_text(encoding="utf-8")
    before = unsupported_characters(raw)
    if before == 0:
        return 0
    text = normalize_copy_style(raw)
    path.write_text(text, encoding="utf-8")
    after = unsupported_characters(path.read_text(encoding="utf-8"))
    return before - after


def main() -> None:
    removed = 0
    for path in sorted(CATALOG.glob("*.json")):
        removed += process_json_file(path)

    for path in sorted(BUILDER_DATA.glob("*.json")):
        removed += process_json_file(path)

    for path in sorted(I18N.rglob("*.json")):
        removed += process_i18n_file(path)

    scripts_translations = ROOT / "scripts" / "catalog_en_translations.json"
    if scripts_translations.exists():
        removed += process_json_file(scripts_translations)

    print(f"Normalized {removed} typography characters in catalog/i18n copy.")


if __name__ == "__main__":
    main()
