#!/usr/bin/env python3
"""One-off generator for tier_a_expansion_v4.json — hand-authored copy per family/suffix."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent / "data" / "tier_a_expansion_v4.json"

FAMILY = {
    "Water": {
        "topic_ru": "водная поверхность",
        "topic_en": "water surface",
        "impact_ru": "Заметно на уровнях с океаном, реками и дождём.",
        "impact_en": "Noticeable on levels with ocean, rivers, and rain.",
        "cat": "Rendering",
    },
    "Star": {
        "topic_ru": "звёздное небо",
        "topic_en": "starfield sky",
        "impact_ru": "Низкая–средняя нагрузка ночью на открытом небе.",
        "impact_en": "Low–medium load at night under open sky.",
        "cat": "Rendering",
    },
    "Sun": {
        "topic_ru": "солнце и небесный свет",
        "topic_en": "sun and skylight",
        "impact_ru": "Сильно влияет на FPS на открытых картах.",
        "impact_en": "Strong FPS impact on open maps.",
        "cat": "Rendering",
    },
    "Tree": {
        "topic_ru": "деревья и листва",
        "topic_en": "trees and foliage",
        "impact_ru": "Средняя–высокая в лесах и джунглях.",
        "impact_en": "Medium–high in forests and jungles.",
        "cat": "Rendering",
    },
    "World": {
        "topic_ru": "глобальные настройки открытого мира",
        "topic_en": "open-world global settings",
        "impact_ru": "Влияет на дальность и детализацию больших карт.",
        "impact_en": "Affects range and detail on large maps.",
        "cat": "Rendering",
    },
    "Static": {
        "topic_ru": "статическая геометрия и инстансы",
        "topic_en": "static geometry and instancing",
        "impact_ru": "Средняя нагрузка на плотных уровнях со статикой.",
        "impact_en": "Medium load on dense static-heavy levels.",
        "cat": "Rendering",
    },
}

SUFFIX = {
    "Enable": {
        "title_ru": "{topic}: вкл/выкл",
        "title_en": "{topic}: enable",
        "desc_ru": "0 — эффект {topic} отключён или упрощён (экономия FPS). 1 — полная отрисовка {topic} как задумано в игре.",
        "desc_en": "0 — {topic} effect disabled or simplified (FPS savings). 1 — full {topic} rendering as intended.",
        "hint_ru": "0 — FPS, 1 — качество",
        "hint_en": "0 — FPS, 1 — quality",
        "min": "0",
        "max": "1",
        "type": "int",
    },
    "Quality": {
        "title_ru": "{topic}: качество",
        "title_en": "{topic}: quality",
        "desc_ru": "Общий уровень качества {topic}. 0 — грубо и быстро. 4 — максимум деталей, но тяжелее для GPU.",
        "desc_en": "Overall {topic} quality. 0 — coarse and fast. 4 — maximum detail but heavier on GPU.",
        "hint_ru": "2 — баланс, 0 — экономия",
        "hint_en": "2 — balance, 0 — save",
        "min": "0",
        "max": "4",
        "type": "int",
    },
    "Distance": {
        "title_ru": "{topic}: дальность",
        "title_en": "{topic}: distance",
        "desc_ru": "На каком расстоянии {topic} перестаёт влиять на картинку. Меньше — экономия FPS, эффект «обрывается» ближе. Больше — видно дальше.",
        "desc_en": "Distance at which {topic} stops affecting the image. Shorter — FPS savings, effect cuts off sooner. Longer — visible farther.",
        "hint_ru": "уменьшить для open world FPS",
        "hint_en": "lower for open-world FPS",
        "min": "100",
        "max": "100000",
        "type": "float",
    },
    "Intensity": {
        "title_ru": "{topic}: яркость",
        "title_en": "{topic}: intensity",
        "desc_ru": "Глобальный множитель яркости {topic}. 1 — как в игре. Выше — пересвет, ниже — тусклее.",
        "desc_en": "Global brightness multiplier for {topic}. 1 — as in game. Higher — blown out, lower — dimmer.",
        "hint_ru": "1 — норма",
        "hint_en": "1 — normal",
        "min": "0",
        "max": "4",
        "type": "float",
    },
    "Max": {
        "title_ru": "{topic}: верхний предел",
        "title_en": "{topic}: max clamp",
        "desc_ru": "Ограничивает максимальный вклад {topic}. Полезно, если эффект в ini слишком сильный.",
        "desc_en": "Caps maximum {topic} contribution. Useful if the effect is too strong in ini.",
        "hint_ru": "уменьшить при пересвете",
        "hint_en": "lower if overbright",
        "min": "0",
        "max": "10",
        "type": "float",
    },
    "Min": {
        "title_ru": "{topic}: нижний предел",
        "title_en": "{topic}: min clamp",
        "desc_ru": "Минимальный вклад {topic} — слабый эффект не опускается ниже этого значения.",
        "desc_en": "Minimum {topic} contribution — weak effect won't go below this.",
        "hint_ru": "0 — прозрачный минимум",
        "hint_en": "0 — transparent minimum",
        "min": "0",
        "max": "1",
        "type": "float",
    },
    "Resolution": {
        "title_ru": "{topic}: разрешение",
        "title_en": "{topic}: resolution",
        "desc_ru": "Разрешение буфера/текстур для {topic}. Ниже — быстрее, но «мыльнее». Выше — чётче, но тяжелее.",
        "desc_en": "Buffer/texture resolution for {topic}. Lower — faster but blurrier. Higher — sharper but heavier.",
        "hint_ru": "512–1024 — баланс",
        "hint_en": "512–1024 — balance",
        "min": "128",
        "max": "4096",
        "type": "int",
    },
    "Scale": {
        "title_ru": "{topic}: масштаб",
        "title_en": "{topic}: scale",
        "desc_ru": "Масштабирует размер/амplitude эффекта {topic}. 1 — по умолчанию.",
        "desc_en": "Scales size/amplitude of {topic}. 1 — default.",
        "hint_ru": "1 — стандарт",
        "hint_en": "1 — default",
        "min": "0.1",
        "max": "4",
        "type": "float",
    },
    "Size": {
        "title_ru": "{topic}: размер источника",
        "title_en": "{topic}: source size",
        "desc_ru": "Виртуальный размер источника для {topic} — влияет на мягкость переходов и теней.",
        "desc_en": "Virtual source size for {topic} — affects transition softness and shadows.",
        "hint_ru": "больше — мягче",
        "hint_en": "higher — softer",
        "min": "0",
        "max": "100",
        "type": "float",
    },
    "Threshold": {
        "title_ru": "{topic}: порог отсечения",
        "title_en": "{topic}: cutoff threshold",
        "desc_ru": "Отсекает слабый вклад {topic} ниже порога — экономия GPU. Выше — агрессивнее FPS, ниже — плавнее.",
        "desc_en": "Culls weak {topic} below threshold — GPU savings. Higher — more aggressive FPS, lower — smoother.",
        "hint_ru": "не трогать без причины",
        "hint_en": "leave unless needed",
        "min": "0",
        "max": "1",
        "type": "float",
    },
    "Bias": {
        "title_ru": "{topic}: bias сэмплинга",
        "title_en": "{topic}: sampling bias",
        "desc_ru": "Смещение при выборке {topic} — убирает артефакты, но слишком большой bias даёт «плавающий» эффект.",
        "desc_en": "Sampling bias for {topic} — removes artifacts but too much causes floating effect.",
        "hint_ru": "только при артефактах",
        "hint_en": "only for artifacts",
        "min": "0",
        "max": "10",
        "type": "float",
    },
    "Count": {
        "title_ru": "{topic}: число сэмплов",
        "title_en": "{topic}: sample count",
        "desc_ru": "Сколько сэмплов используется для {topic}. Больше — плавнее, меньше — быстрее.",
        "desc_en": "Samples used for {topic}. More — smoother, fewer — faster.",
        "hint_ru": "8–16 — баланс",
        "hint_en": "8–16 — balance",
        "min": "1",
        "max": "64",
        "type": "int",
    },
}

CUSTOM = {
    "r.Water.WaterInfo": {
        "title": "Вода: доп. данные WaterInfo",
        "title_en": "Water: WaterInfo data",
        "description": "Включает расширенный проход WaterInfo для водных поверхностей UE5 — глубина, берег, пена. 0 — упрощённая вода. 1 — полный pipeline, если игра его использует.",
        "description_en": "Enables extended WaterInfo pass for UE5 water — depth, shoreline, foam. 0 — simplified water. 1 — full pipeline if the game uses it.",
        "impact": "Средняя–высокая на уровнях с океаном.",
        "impact_en": "Medium–high on ocean levels.",
        "min": "0",
        "max": "1",
        "value_hint": "1 — если игра поддерживает WaterInfo",
        "value_hint_en": "1 — if game supports WaterInfo",
        "category": "Rendering",
        "value_type": "int",
    },
    "r.Water.WaveAmplitude": {
        "title": "Вода: амплитуда волн",
        "title_en": "Water wave amplitude",
        "description": "Насколько высокие волны на водной поверхности. 0 — зеркальная гладь. Выше — бурнее море, но может вызывать укачивание камеры и нагрузку на симуляцию.",
        "description_en": "How tall water waves are. 0 — mirror-flat surface. Higher — rougher sea but more simulation load and camera bob.",
        "impact": "Средняя на водных уровнях.",
        "impact_en": "Medium on water levels.",
        "min": "0",
        "max": "10",
        "value_hint": "1 — спокойная вода, 3+ — шторм",
        "value_hint_en": "1 — calm, 3+ — storm",
        "category": "Rendering",
        "value_type": "float",
    },
    "r.Water.WaveFrequency": {
        "title": "Вода: частота волн",
        "title_en": "Water wave frequency",
        "description": "Как часто волны следуют друг за другом. Ниже — длинные медленные волны. Выше — частая рябь, живее вода, чуть тяжелее для GPU.",
        "description_en": "How often waves follow each other. Lower — long slow swells. Higher — frequent ripples, livelier water, slightly heavier GPU.",
        "impact": "Низкая–средняя.",
        "impact_en": "Low–medium.",
        "min": "0.1",
        "max": "10",
        "value_hint": "0.5–2 — естественно",
        "value_hint_en": "0.5–2 — natural",
        "category": "Rendering",
        "value_type": "float",
    },
}


def fmt(template: str, topic_ru: str, topic_en: str) -> str:
    return template.format(topic=topic_ru if "{" in template and "topic" in template else topic_en)


def build_entry(key: str) -> dict:
    if key in CUSTOM:
        return CUSTOM[key]
    _, family, suffix = key.split(".", 2) if key.count(".") == 2 else (None, None, None)
    if key.count(".") == 2:
        prefix, family, suffix = key.split(".")
    else:
        raise ValueError(key)
    meta = FAMILY[family]
    suf = SUFFIX[suffix]
    topic_ru = meta["topic_ru"]
    topic_en = meta["topic_en"]
    return {
        "title": suf["title_ru"].format(topic=topic_ru),
        "title_en": suf["title_en"].format(topic=topic_en),
        "description": suf["desc_ru"].format(topic=topic_ru),
        "description_en": suf["desc_en"].format(topic=topic_en),
        "impact": meta["impact_ru"],
        "impact_en": meta["impact_en"],
        "min": suf["min"],
        "max": suf["max"],
        "value_hint": suf["hint_ru"],
        "value_hint_en": suf["hint_en"],
        "category": meta["cat"],
        "value_type": suf["type"],
    }


KEYS = [
    "r.Star.Distance", "r.Star.Enable", "r.Star.Intensity", "r.Star.Max", "r.Star.Min",
    "r.Star.Quality", "r.Star.Resolution", "r.Star.Scale", "r.Star.Size", "r.Star.Threshold",
    "r.Static.Bias", "r.Static.Count", "r.Static.Distance", "r.Static.Enable", "r.Static.Intensity", "r.Static.Max",
    "r.Static.Min", "r.Static.Quality", "r.Static.Resolution", "r.Static.Scale", "r.Static.Size",
    "r.Static.Threshold",
    "r.Sun.Bias", "r.Sun.Count", "r.Sun.Distance", "r.Sun.Enable", "r.Sun.Intensity", "r.Sun.Max", "r.Sun.Min",
    "r.Sun.Quality", "r.Sun.Resolution", "r.Sun.Scale", "r.Sun.Size", "r.Sun.Threshold",
    "r.Tree.Bias", "r.Tree.Count", "r.Tree.Distance", "r.Tree.Enable", "r.Tree.Intensity",
    "r.Tree.Max", "r.Tree.Min", "r.Tree.Quality", "r.Tree.Resolution", "r.Tree.Scale",
    "r.Tree.Size", "r.Tree.Threshold",
    "r.Water.Bias", "r.Water.Count", "r.Water.Distance", "r.Water.Enable", "r.Water.Intensity",
    "r.Water.Max", "r.Water.Min", "r.Water.Quality", "r.Water.Resolution", "r.Water.Scale",
    "r.Water.Size", "r.Water.Threshold", "r.Water.WaterInfo", "r.Water.WaveAmplitude",
    "r.Water.WaveFrequency",
    "r.World.Bias", "r.World.Count", "r.World.Distance", "r.World.Enable", "r.World.Intensity", "r.World.Max",
    "r.World.Min", "r.World.Quality", "r.World.Resolution", "r.World.Scale", "r.World.Size",
    "r.World.Threshold",
]


def main() -> None:
    out = {key: build_entry(key) for key in KEYS}
    ROOT.write_text(json.dumps(out, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"wrote {len(out)} entries -> {ROOT}")


if __name__ == "__main__":
    main()
