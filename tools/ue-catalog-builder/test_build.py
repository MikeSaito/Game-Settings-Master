"""Unit tests for UE catalog builder."""
from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(Path(__file__).resolve().parent))

from build import (  # noqa: E402
    apply_gus_registry,
    apply_sg_registry,
    apply_tier_overlay,
    humanize_key,
    load_display_overrides,
    merge_entries,
    MergedEntry,
    parse_engine_ini,
    parse_ini,
    parse_scalability_ini,
)
from extract.gus_from_header import extract_from_files as extract_gus_from_files  # noqa: E402
from extract.sg_from_cpp import extract_from_file as extract_sg_from_file  # noqa: E402


class ParserTests(unittest.TestCase):
    def test_parse_mini_base_engine(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "BaseEngine.ini"
            path.write_text(
                "[SystemSettings]\n"
                "r.ViewDistanceScale=1.0\n"
                "r.ShadowQuality=5\n"
                "\n"
                "[ConsoleVariables]\n"
                "r.OneFrameThreadLag=1\n",
                encoding="utf-8",
            )
            entries = parse_engine_ini("5.4", path)
            keys = {e.key for e in entries}
            self.assertIn("r.ViewDistanceScale", keys)
            self.assertIn("r.OneFrameThreadLag", keys)
            self.assertEqual(len(entries), 3)

    def test_parse_scalability_tiers(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "BaseScalability.ini"
            path.write_text(
                "[ScalabilityGroups]\n"
                "sg.ShadowQuality=3\n"
                "\n"
                "[ShadowQuality@2]\n"
                "r.ShadowQuality=2\n"
                "r.Shadow.MaxResolution=1024\n",
                encoding="utf-8",
            )
            entries, tiers = parse_scalability_ini("5.4", path)
            self.assertTrue(any(e.key == "sg.ShadowQuality" for e in entries))
            self.assertEqual(len(tiers), 1)
            self.assertEqual(tiers[0].index, 2)
            self.assertIn("r.ShadowQuality", tiers[0].cvars)

    def test_merge_versions_tracks_ue4_ue5(self) -> None:
        from build import ParsedEntry

        by_version = {
            "4.27": [
                ParsedEntry(
                    key="r.Foo",
                    default_value="1",
                    section="SystemSettings",
                    file="Engine.ini",
                    source="BaseEngine.ini",
                )
            ],
            "5.4": [
                ParsedEntry(
                    key="r.Foo",
                    default_value="2",
                    section="SystemSettings",
                    file="Engine.ini",
                    source="BaseEngine.ini",
                ),
                ParsedEntry(
                    key="r.Nanite",
                    default_value="1",
                    section="SystemSettings",
                    file="Engine.ini",
                    source="BaseEngine.ini",
                ),
            ],
        }
        merged = merge_entries(by_version, ["4.27", "5.4"])
        foo = next(m for m in merged if m.key == "r.Foo")
        self.assertTrue(foo.ue4 and foo.ue5)
        nanite = next(m for m in merged if m.key == "r.Nanite")
        self.assertFalse(nanite.ue4)
        self.assertTrue(nanite.ue5)

    def test_extract_sg_from_scalability_cpp(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "Scalability.cpp"
            path.write_text(
                'static TAutoConsoleVariable<float> CVarResolutionQuality(TEXT("sg.ResolutionQuality"), 0.0f);\n'
                'static TAutoConsoleVariable<int32> CVarShadowQuality(TEXT("sg.ShadowQuality"), 3);\n'
                'static TAutoConsoleVariable<int32> CVarShadowQuality_NumLevels(TEXT("sg.ShadowQuality.NumLevels"), 5);\n'
                'static TAutoConsoleVariable<float> CVarTest(TEXT("sg.Test.CPUPerfIndexOverride"), 0.0f);\n',
                encoding="utf-8",
            )
            data = extract_sg_from_file(path)
            self.assertEqual(data["keys"], ["sg.ResolutionQuality", "sg.ShadowQuality"])
            self.assertEqual(data["metadata"]["num_levels"]["sg.ShadowQuality"], 5)

    def test_extract_gus_config_fields(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            header = Path(tmp) / "GameUserSettings.h"
            cpp = Path(tmp) / "GameUserSettings.cpp"
            header.write_text(
                "UPROPERTY(config)\n"
                "bool bUseVSync;\n"
                "/** Frame rate cap */\n"
                "UPROPERTY(config)\n"
                "float FrameRateLimit;\n"
                "UPROPERTY(globalconfig)\n"
                "bool bUseDesiredScreenHeight;\n",
                encoding="utf-8",
            )
            cpp.write_text(
                "void UGameUserSettings::SetToDefaults()\n"
                "{\n"
                "    FrameRateLimit = 0.0f;\n"
                "    bUseVSync = false;\n"
                "}\n",
                encoding="utf-8",
            )
            fields = {field["key"]: field for field in extract_gus_from_files(header, cpp)}
            self.assertEqual(fields["bUseVSync"]["value_type"], "bool")
            self.assertEqual(fields["FrameRateLimit"]["default"], "0.0")
            self.assertEqual(fields["bUseDesiredScreenHeight"]["config_scope"], "globalconfig")

    def test_generated_registries_feed_builder(self) -> None:
        merged: dict[str, MergedEntry] = {}
        sg_count = apply_sg_registry(merged, ["4.27", "5.4"])
        gus_count = apply_gus_registry(merged, ["4.27", "5.4"], set())
        if sg_count:
            shadow = merged["sg.shadowquality"]
            self.assertEqual(shadow.file, "GameUserSettings.ini")
            self.assertEqual(shadow.section, "ScalabilityGroups")
        if gus_count:
            vsync = merged["busevsync"]
            self.assertEqual(vsync.file, "GameUserSettings.ini")
            self.assertEqual(vsync.section, "/Script/Engine.GameUserSettings")

    def test_humanize_key_splits_camelcase_and_acronyms(self) -> None:
        self.assertEqual(
            humanize_key("r.Lumen.FinalGatherQuality"),
            "Lumen · Final · Gather · Quality",
        )
        self.assertEqual(
            humanize_key("r.TemporalAA.Upsampling"),
            "Temporal · AA · Upsampling",
        )
        self.assertEqual(
            humanize_key("r.Shadow.Virtual.Enable"),
            "Shadow · Virtual · Enable",
        )

    def test_display_overrides_have_top_priority(self) -> None:
        overrides = load_display_overrides()
        self.assertIn("r.screenpercentage", overrides)
        row = {
            "key": "r.ScreenPercentage",
            "title": "Screen · Percentage",
            "description": "auto",
            "category_guess": "Rendering",
        }
        apply_tier_overlay(row, overrides)
        self.assertEqual(row["title"], "Масштаб внутреннего разрешения")
        self.assertEqual(row["category_guess"], "Display")


class MergeStatsTests(unittest.TestCase):
    def test_curated_scalability_covers_sg_registry(self) -> None:
        registry_path = ROOT / "tools" / "ue-catalog-builder" / "generated" / "sg_registry_merged.json"
        curated_path = ROOT / "src-tauri" / "catalog" / "scalability.json"
        if not registry_path.exists():
            self.skipTest("run tools/ue-catalog-builder/extract/sg_from_cpp.py first")
        registry = json.loads(registry_path.read_text(encoding="utf-8"))
        curated = json.loads(curated_path.read_text(encoding="utf-8"))
        curated_keys = {item["key"] for item in curated}
        missing = [item["key"] for item in registry["keys"] if item["key"] not in curated_keys]
        self.assertEqual(missing, [])

    def test_display_catalog_covers_gus_registry(self) -> None:
        registry_path = ROOT / "tools" / "ue-catalog-builder" / "generated" / "gus_registry_merged.json"
        curated_path = ROOT / "src-tauri" / "catalog" / "display.json"
        if not registry_path.exists():
            self.skipTest("run tools/ue-catalog-builder/extract/gus_from_header.py first")
        registry = json.loads(registry_path.read_text(encoding="utf-8"))
        curated = json.loads(curated_path.read_text(encoding="utf-8"))
        curated_keys = {item["key"] for item in curated}
        missing = [item["key"] for item in registry["keys"] if item["key"] not in curated_keys]
        self.assertEqual(missing, [])

    def test_display_catalog_has_no_corrupted_question_marks(self) -> None:
        curated_path = ROOT / "src-tauri" / "catalog" / "display.json"
        data = json.loads(curated_path.read_text(encoding="utf-8"))
        offenders = []
        for item in data:
            for field in ("title", "description", "impact", "value_hint"):
                value = str(item.get(field) or "")
                if "???" in value or "? enabled" in value or "? disabled" in value:
                    offenders.append((item.get("key"), field, value))
        self.assertEqual(offenders, [])

    def test_generated_index_meets_minimum(self) -> None:
        index_path = ROOT / "src-tauri" / "catalog" / "ue_reference_index.json"
        stats_path = ROOT / "src-tauri" / "catalog" / "generated" / "merge_stats.json"
        self.assertTrue(index_path.exists(), "run npm run catalog:build first")
        data = json.loads(index_path.read_text(encoding="utf-8"))
        stats = json.loads(stats_path.read_text(encoding="utf-8")) if stats_path.exists() else {}
        sources = stats.get("sources") or data.get("sources") or []
        if len(sources) >= 8:
            min_keys = 700
        elif len(sources) > 2:
            min_keys = 1000
        else:
            min_keys = 548
        self.assertGreaterEqual(data.get("schema_version", 1), 2)
        self.assertGreaterEqual(len(data["entries"]), min_keys)
        self.assertGreaterEqual(len(sources), 2)
        if stats.get("sg_registry_count") is not None:
            self.assertGreaterEqual(stats["sg_registry_count"], 12)
        if stats.get("gus_registry_count") is not None:
            self.assertGreaterEqual(stats["gus_registry_count"], 20)


if __name__ == "__main__":
    unittest.main()
