# UE catalog builder

Parses local `BaseEngine.ini` / `BaseScalability.ini` snapshots into `src-tauri/catalog/ue_reference_index.json`.

```powershell
python tools/ue-catalog-builder/extract_sg_from_cpp.py --all-versions
python tools/ue-catalog-builder/extract_gus_from_header.py --all-versions
npm run catalog:build
npm run catalog:test
```

## Layout

- `build.py` — parser, multi-version merge, supplemental CVars
- `extract_sg_from_cpp.py` — extracts official `sg.*` registry from `Scalability.cpp`
- `extract_gus_from_header.py` — extracts `UGameUserSettings` config fields from `GameUserSettings.h`
- `gen_fixtures.py` — regenerate committed sample ini under `tools/ue-reference/fixtures/`
- `data/supplemental_cvars.json` — extra well-known CVars when snapshots are small
- `generated/sg_registry_merged.json` — generated `sg.*` registry with version bounds
- `generated/gus_registry_merged.json` — generated display/user settings registry
- `test_build.py` — unit tests

### Tier overlays

| File | Purpose |
|------|---------|
| `data/tier_a_descriptions.json` | Human RU/EN from curated catalog (~83 keys) |
| `data/tier_b_descriptions.json` | Semi-human for top reference-only CVars (~150 keys) |

Merge order: **curated JSON > tier A > tier B > auto-generated**.

Reference entries include `catalog_recommended: bool` (curated keys, tier A/B, sg.*).
